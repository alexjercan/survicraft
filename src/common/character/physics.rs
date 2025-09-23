//! TODO: Add documentation

use avian3d::{math::AdjustPrecision, prelude::*};
use bevy::{ecs::query::QueryData, prelude::*};

pub mod prelude {
    pub use super::{
        PhysicsCharacterBundle, PhysicsCharacterInput, PhysicsCharacterPlugin,
        CHARACTER_CAPSULE_HEIGHT, CHARACTER_CAPSULE_RADIUS,
    };
}

#[cfg(feature = "debug")]
use self::debug::PlayerDebugPlugin;

pub const CHARACTER_CAPSULE_RADIUS: f32 = 0.5;
pub const CHARACTER_CAPSULE_HEIGHT: f32 = 1.0;

pub struct PhysicsCharacterPlugin;

impl Plugin for PhysicsCharacterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PhysicsCharacterInput>();

        #[cfg(feature = "debug")]
        app.add_plugins(PlayerDebugPlugin);

        app.add_systems(
            Update,
            (
                update_grounded,
                handle_character_actions,
                apply_movement_damping,
            )
                .chain(),
        );
    }
}

/// A marker component indicating that an entity is using a character controller.
#[derive(Component)]
pub struct PhysicsCharacterController;

/// A marker component indicating that an entity is on the ground.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct PhysicsCharacterGrounded;

#[derive(Bundle)]
pub struct PhysicsCharacterBundle {
    controller: PhysicsCharacterController,
    collider: Collider,
    rigid_body: RigidBody,
    external_force: ExternalForce,
    external_impulse: ExternalImpulse,
    lock_axes: LockedAxes,
    friction: Friction,
    ground_caster: ShapeCaster,
}

impl Default for PhysicsCharacterBundle {
    fn default() -> Self {
        let collider = Collider::capsule(CHARACTER_CAPSULE_RADIUS, CHARACTER_CAPSULE_HEIGHT);
        let mut caster_shape = collider.clone();
        caster_shape.set_scale(Vec3::ONE * 0.99, 10);

        Self {
            controller: PhysicsCharacterController,
            collider,
            rigid_body: RigidBody::Dynamic,
            external_force: ExternalForce::ZERO.with_persistence(false),
            external_impulse: ExternalImpulse::ZERO.with_persistence(false),
            lock_axes: LockedAxes::default()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z(),
            friction: Friction::new(0.0).with_combine_rule(CoefficientCombine::Min),
            ground_caster: ShapeCaster::new(caster_shape, Vec3::ZERO, Quat::default(), Dir3::NEG_Y)
                .with_max_distance(0.2),
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
pub struct PhysicsCharacterInput {
    pub move_axis: Vec2,
    pub jump: bool,
}

fn update_grounded(
    mut commands: Commands,
    mut q_controller: Query<(Entity, &ShapeHits), With<PhysicsCharacterController>>,
) {
    for (entity, hits) in &mut q_controller {
        // The character is grounded if the shape caster has a hit with a normal
        // that isn't too steep.
        let is_grounded = hits.iter().any(|_| true);

        if is_grounded {
            commands.entity(entity).insert(PhysicsCharacterGrounded);
        } else {
            commands.entity(entity).remove::<PhysicsCharacterGrounded>();
        }
    }
}

fn handle_character_actions(
    time: Res<Time>,
    mut q_player: Query<(&PhysicsCharacterInput, CharacterQuery)>,
) {
    for (input, mut character) in &mut q_player {
        apply_character_action(&time, input, &mut character);
    }
}

#[derive(QueryData)]
#[query_data(mutable, derive(Debug))]
struct CharacterQuery {
    linear_velocity: &'static mut LinearVelocity,
    position: &'static Position,
    rotation: &'static Rotation,
    entity: Entity,
    grounded: Has<PhysicsCharacterGrounded>,
}

/// Apply the character actions `action_state` to the character entity `character`.
fn apply_character_action(
    time: &Res<Time>,
    input: &PhysicsCharacterInput,
    character: &mut CharacterQueryItem,
) {
    // TODO: unhardcode these values
    const MAX_ACCELERATION: f32 = 30.0;
    const JUMP_FORCE: f32 = 7.0;

    let delta_time = time.delta_secs_f64().adjust_precision();

    // === MOVEMENT ===
    let move_input = input.move_axis.clamp_length_max(1.0);
    let local_move = Vec3::new(move_input.x, 0.0, -move_input.y);
    let world_move = character.rotation.0 * local_move;

    character.linear_velocity.x += world_move.x * MAX_ACCELERATION * delta_time;
    character.linear_velocity.z += world_move.z * MAX_ACCELERATION * delta_time;

    // === JUMPING ===
    if input.jump && character.grounded {
        character.linear_velocity.y = JUMP_FORCE;
    }
}

fn apply_movement_damping(mut query: Query<&mut LinearVelocity, With<PhysicsCharacterController>>) {
    let damping_factor = 0.92;
    for mut linear_velocity in &mut query {
        linear_velocity.x *= damping_factor;
        linear_velocity.z *= damping_factor;
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

    fn log_player_character_state(q_player: Query<CharacterQuery, With<PhysicsCharacterInput>>) {
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
