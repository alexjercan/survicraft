//! TODO: Add documentation

use avian3d::{
    math::{AdjustPrecision, Scalar, Vector},
    prelude::*,
};
use bevy::{ecs::query::QueryData, prelude::*};

use super::{components::*, constants::*};

pub mod prelude {
    pub use super::{
        ControllerGravity, JumpImpulse, KinematicCharacterBundle, KinematicCharacterGrounded,
        KinematicCharacterPlugin, MaxSlopeAngle, MovementAcceleration, MovementBundle,
        MovementDampingFactor,
    };
}

#[cfg(feature = "debug")]
use self::debug::PlayerDebugPlugin;

pub struct KinematicCharacterPlugin;

impl Plugin for KinematicCharacterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CharacterInput>()
            .register_type::<MovementAcceleration>()
            .register_type::<MovementDampingFactor>()
            .register_type::<JumpImpulse>()
            .register_type::<JumpTimer>()
            .register_type::<GroundedCooldown>()
            .register_type::<ControllerGravity>()
            .register_type::<MaxSlopeAngle>();

        #[cfg(feature = "debug")]
        app.add_plugins(PlayerDebugPlugin);

        app.add_systems(
            Update,
            (
                update_grounded,
                apply_gravity,
                handle_character_actions,
                apply_movement_damping,
            )
                .chain(),
        )
        .add_systems(
            // Run collision handling after collision detection.
            //
            // NOTE: The collision implementation here is very basic and a bit buggy.
            //       A collide-and-slide algorithm would likely work better.
            PhysicsSchedule,
            kinematic_controller_collisions.in_set(NarrowPhaseSet::Last),
        );
    }
}

/// The acceleration used for character movement.
#[derive(Component, Clone, Copy, Debug, Deref, DerefMut, Reflect)]
pub struct MovementAcceleration(Scalar);

/// The damping factor used for slowing down movement.
#[derive(Component, Clone, Copy, Debug, Deref, DerefMut, Reflect)]
pub struct MovementDampingFactor(Scalar);

/// The strength of a jump.
#[derive(Component, Clone, Copy, Debug, Deref, DerefMut, Reflect)]
pub struct JumpImpulse(Scalar);

/// A timer used to allow late jumps when Grounded flickers.
#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
pub struct JumpTimer(Timer);

/// A cooldown timer used to prevent immediate re-grounding after a jump.
#[derive(Component, Default, Debug, Deref, DerefMut, Reflect)]
pub struct GroundedCooldown(Timer);

/// The gravitational acceleration used for a character controller.
#[derive(Component, Clone, Copy, Debug, Deref, DerefMut, Reflect)]
pub struct ControllerGravity(Vector);

/// The maximum angle a slope can have for a character controller
/// to be able to climb and jump. If the slope is steeper than this angle,
/// the character will slide down.
#[derive(Component, Clone, Copy, Debug, Deref, DerefMut, Reflect)]
pub struct MaxSlopeAngle(Scalar);

/// A marker component indicating that an entity is on the ground.
#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
pub struct KinematicCharacterGrounded;

/// A bundle that contains the components needed for a basic
/// kinematic character controller.
#[derive(Bundle)]
pub struct KinematicCharacterBundle {
    controller: CharacterController,
    body: RigidBody,
    collider: Collider,
    ground_caster: ShapeCaster,
    gravity: ControllerGravity,
    movement: MovementBundle,
}

impl Default for KinematicCharacterBundle {
    fn default() -> Self {
        let collider = Collider::capsule(CHARACTER_CAPSULE_RADIUS, CHARACTER_CAPSULE_HEIGHT);
        let mut caster_shape = collider.clone();
        caster_shape.set_scale(Vec3::ONE * 0.99, 10);

        Self {
            controller: CharacterController,
            body: RigidBody::Kinematic,
            collider,
            ground_caster: ShapeCaster::new(caster_shape, Vec3::ZERO, Quat::default(), Dir3::NEG_Y)
                .with_max_distance(0.1),
            gravity: ControllerGravity(Vector::new(0.0, -9.81, 0.0)),
            movement: MovementBundle::default(),
        }
    }
}

impl KinematicCharacterBundle {
    pub fn with_gravity(mut self, gravity: Vector) -> Self {
        self.gravity = ControllerGravity(gravity);
        self
    }

    pub fn with_movement(mut self, movement: MovementBundle) -> Self {
        self.movement = movement;
        self
    }
}

/// A bundle that contains components for character movement.
#[derive(Bundle)]
pub struct MovementBundle {
    acceleration: MovementAcceleration,
    damping: MovementDampingFactor,
    jump_impulse: JumpImpulse,
    jump_timer: JumpTimer,
    ground_cooldown: GroundedCooldown,
    max_slope_angle: MaxSlopeAngle,
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self {
            acceleration: MovementAcceleration(30.0),
            damping: MovementDampingFactor(0.92),
            jump_impulse: JumpImpulse(7.0),
            jump_timer: JumpTimer(Timer::from_seconds(0.1, TimerMode::Once)),
            ground_cooldown: GroundedCooldown(Timer::from_seconds(0.1, TimerMode::Once)),
            max_slope_angle: MaxSlopeAngle(Scalar::to_radians(80.0)),
        }
    }
}

impl MovementBundle {
    pub fn with_acceleration(mut self, accel: Scalar) -> Self {
        self.acceleration = MovementAcceleration(accel);
        self
    }

    pub fn with_damping(mut self, damping: Scalar) -> Self {
        self.damping = MovementDampingFactor(damping);
        self
    }

    pub fn with_jump_impulse(mut self, impulse: Scalar) -> Self {
        self.jump_impulse = JumpImpulse(impulse);
        self
    }

    pub fn with_jump_timer(mut self, secs: f32) -> Self {
        self.jump_timer = JumpTimer(Timer::from_seconds(secs, TimerMode::Once));
        self
    }

    pub fn with_max_slope_angle(mut self, angle: Scalar) -> Self {
        self.max_slope_angle = MaxSlopeAngle(angle);
        self
    }
}

fn update_grounded(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<
        (Entity, &ShapeHits, &Rotation, &mut GroundedCooldown, Option<&MaxSlopeAngle>),
        With<CharacterController>,
    >,
) {
    for (entity, hits, rotation, mut grounded_cooldown, max_slope_angle) in &mut query {
        grounded_cooldown.tick(time.delta());
        if grounded_cooldown.finished() {
            // The cooldown has finished, so we can check for grounded state normally.
        } else {
            // Still in cooldown, so we are not grounded.
            commands
                .entity(entity)
                .remove::<KinematicCharacterGrounded>();
            continue;
        }

        // The character is grounded if the shape caster has a hit with a normal
        // that isn't too steep.
        let is_grounded = hits.iter().any(|hit| {
            if let Some(angle) = max_slope_angle {
                (rotation * -hit.normal2).angle_between(Vector::Y).abs() <= angle.0
            } else {
                true
            }
        });

        if is_grounded {
            commands.entity(entity).insert(KinematicCharacterGrounded);
        } else {
            commands
                .entity(entity)
                .remove::<KinematicCharacterGrounded>();
        }
    }
}

fn handle_character_actions(
    time: Res<Time>,
    mut q_player: Query<(&CharacterInput, CharacterQuery)>,
) {
    for (input, mut character) in &mut q_player {
        apply_character_action(&time, input, &mut character);
    }
}

#[derive(QueryData)]
#[query_data(mutable, derive(Debug))]
struct CharacterQuery {
    entity: Entity,
    acceleration: &'static MovementAcceleration,
    jump_impulse: &'static JumpImpulse,
    jump_timer: &'static mut JumpTimer,
    grounded_cooldown: &'static mut GroundedCooldown,
    position: &'static Position,
    rotation: &'static Rotation,
    linear_velocity: &'static mut LinearVelocity,
    grounded: Has<KinematicCharacterGrounded>,
}

fn apply_character_action(
    time: &Res<Time>,
    input: &CharacterInput,
    character: &mut CharacterQueryItem,
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_secs_f64().adjust_precision();
    character.jump_timer.tick(time.delta());

    // === MOVEMENT ===
    let move_input = input.move_axis.clamp_length_max(1.0);
    let local_move = Vec3::new(move_input.x, 0.0, -move_input.y);
    let world_move = character.rotation.0 * local_move;

    character.linear_velocity.x += world_move.x * (**character.acceleration) * delta_time;
    character.linear_velocity.z += world_move.z * (**character.acceleration) * delta_time;

    // === JUMPING ===
    if character.grounded {
        character.jump_timer.reset();
    }

    if input.jump && !character.jump_timer.finished() {
        character.linear_velocity.y = character.jump_impulse.0;
        character.grounded_cooldown.reset();
    }
}

fn apply_gravity(
    time: Res<Time>,
    mut controllers: Query<(
        &ControllerGravity,
        &mut LinearVelocity,
        Has<KinematicCharacterGrounded>,
    )>,
) {
    let dt = time.delta_secs_f64().adjust_precision();

    for (gravity, mut velocity, grounded) in &mut controllers {
        if grounded {
            // Snap down a little to stick to ground
            velocity.y = velocity.y.min(0.0);
            velocity.y -= 2.0 * dt; // small extra downward force
        } else {
            **velocity += **gravity * dt;
        }
    }
}

fn apply_movement_damping(mut query: Query<(&MovementDampingFactor, &mut LinearVelocity)>) {
    for (damping_factor, mut linear_velocity) in &mut query {
        // We could use `LinearDamping`, but we don't want to dampen movement along the Y axis
        linear_velocity.x *= **damping_factor;
        linear_velocity.z *= **damping_factor;
    }
}

/// Kinematic bodies do not get pushed by collisions by default,
/// so it needs to be done manually.
///
/// This system handles collision response for kinematic character controllers
/// by pushing them along their contact normals by the current penetration depth,
/// and applying velocity corrections in order to snap to slopes, slide along walls,
/// and predict collisions using speculative contacts.
#[allow(clippy::type_complexity)]
fn kinematic_controller_collisions(
    collisions: Collisions,
    bodies: Query<&RigidBody>,
    collider_rbs: Query<&ColliderOf, Without<Sensor>>,
    mut character_controllers: Query<
        (&mut Position, &mut LinearVelocity, Option<&MaxSlopeAngle>),
        (With<RigidBody>, With<CharacterController>),
    >,
    time: Res<Time>,
) {
    // Iterate through collisions and move the kinematic body to resolve penetration
    for contacts in collisions.iter() {
        // Get the rigid body entities of the colliders (colliders could be children)
        let Ok([&ColliderOf { body: rb1 }, &ColliderOf { body: rb2 }]) =
            collider_rbs.get_many([contacts.collider1, contacts.collider2])
        else {
            continue;
        };

        // Get the body of the character controller and whether it is the first
        // or second entity in the collision.
        let is_first: bool;

        let character_rb: RigidBody;
        let is_other_dynamic: bool;

        let (mut position, mut linear_velocity, max_slope_angle) =
            if let Ok(character) = character_controllers.get_mut(rb1) {
                is_first = true;
                character_rb = *bodies.get(rb1).unwrap();
                is_other_dynamic = bodies.get(rb2).is_ok_and(|rb| rb.is_dynamic());
                character
            } else if let Ok(character) = character_controllers.get_mut(rb2) {
                is_first = false;
                character_rb = *bodies.get(rb2).unwrap();
                is_other_dynamic = bodies.get(rb1).is_ok_and(|rb| rb.is_dynamic());
                character
            } else {
                continue;
            };

        // This system only handles collision response for kinematic character controllers.
        if !character_rb.is_kinematic() {
            continue;
        }

        // Iterate through contact manifolds and their contacts.
        // Each contact in a single manifold shares the same contact normal.
        for manifold in contacts.manifolds.iter() {
            let normal = if is_first {
                -manifold.normal
            } else {
                manifold.normal
            };

            let mut deepest_penetration: Scalar = Scalar::MIN;

            // Solve each penetrating contact in the manifold.
            for contact in manifold.points.iter() {
                if contact.penetration > 0.0 {
                    position.0 += normal * (contact.penetration + 0.01); // bias outward
                }
                deepest_penetration = deepest_penetration.max(contact.penetration);
            }

            // For now, this system only handles velocity corrections for collisions against static geometry.
            if is_other_dynamic {
                continue;
            }

            // Determine if the slope is climbable or if it's too steep to walk on.
            let slope_angle = normal.angle_between(Vector::Y);
            let climbable = max_slope_angle.is_some_and(|angle| slope_angle.abs() <= angle.0);

            if deepest_penetration > 0.0 {
                // If the slope is climbable, snap the velocity so that the character
                // up and down the surface smoothly.
                if climbable {
                    // Project velocity onto the slope plane instead of adjusting Y separately
                    let slope_normal = normal.normalize();
                    let projected = linear_velocity.reject_from_normalized(slope_normal);

                    // Blend between original and projected velocity to avoid sticky/laggy edges
                    **linear_velocity = linear_velocity.lerp(projected, 0.5);
                } else {
                    // The character is intersecting an unclimbable object, like a wall.
                    // We want the character to slide along the surface, similarly to
                    // a collide-and-slide algorithm.

                    // Don't apply an impulse if the character is moving away from the surface.
                    if linear_velocity.dot(normal) > 0.0 {
                        continue;
                    }

                    // Slide along the surface, rejecting the velocity along the contact normal.
                    let impulse = linear_velocity.reject_from_normalized(normal);
                    **linear_velocity = impulse;
                }
            } else {
                // The character is not yet intersecting the other object,
                // but the narrow phase detected a speculative collision.
                //
                // We need to push back the part of the velocity
                // that would cause penetration within the next frame.

                let normal_speed = linear_velocity.dot(normal);

                // Don't apply an impulse if the character is moving away from the surface.
                if normal_speed > 0.0 {
                    continue;
                }

                // Compute the impulse to apply.
                let impulse_magnitude =
                    normal_speed - (deepest_penetration / time.delta_secs_f64().adjust_precision());
                let mut impulse = impulse_magnitude * normal;

                // Apply the impulse differently depending on the slope angle.
                if climbable {
                    // Avoid sliding down slopes.
                    linear_velocity.y -= impulse.y.min(0.0);
                } else {
                    // Avoid climbing up walls.
                    impulse.y = impulse.y.max(0.0);
                    **linear_velocity -= impulse;
                }
            }
        }
    }
}

#[cfg(feature = "debug")]
mod debug {
    use super::*;

    pub struct PlayerDebugPlugin;
    impl Plugin for PlayerDebugPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(Update, log_player_character_state);
        }
    }

    fn log_player_character_state(q_player: Query<CharacterQuery, With<CharacterInput>>) {
        for character in &q_player {
            trace!(
                "PlayerController {:?}: LinearVelocity={:?}, Position={:?}, Rotation={:?}",
                character.entity,
                character.linear_velocity,
                character.position,
                character.rotation,
            );
        }
    }
}
