//! TODO: Add documentation

use avian3d::prelude::*;
use bevy::{ecs::query::QueryData, prelude::*};

pub mod prelude {
    pub use super::{
        PlayerCharacterController, PlayerCharacterInput, PlayerPlugin, PlayerPluginSet,
        CHARACTER_CAPSULE_HEIGHT, CHARACTER_CAPSULE_RADIUS,
    };
}

pub const CHARACTER_CAPSULE_RADIUS: f32 = 0.5;
pub const CHARACTER_CAPSULE_HEIGHT: f32 = 1.0;

/// Marker component for the player character entity. Spawn this when you
/// want to attach a player bundle and have it be controlled by a player.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerCharacterController;

/// The input component for the player character, which stores the current input state.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct PlayerCharacterInput {
    pub move_axis: Vec2,
    pub jump: bool,
    pub look: Vec2,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerPluginSet;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_spawn_player.in_set(PlayerPluginSet));
        app.add_systems(
            FixedUpdate,
            handle_character_actions.in_set(PlayerPluginSet),
        );
    }
}

fn handle_spawn_player(
    mut commands: Commands,
    q_player: Query<Entity, Added<PlayerCharacterController>>,
) {
    for entity in &q_player {
        debug!("Adding PlayerCharacterInput and CharacterPhysicsBundle to entity {entity:?}");

        commands.entity(entity).insert((
            PlayerCharacterInput::default(),
            CharacterPhysicsBundle::default(),
        ));
    }
}

fn handle_character_actions(
    time: Res<Time>,
    // spatial_query: SpatialQuery,
    mut q_player: Query<(&PlayerCharacterInput, CharacterQuery), With<PlayerCharacterController>>,
) {
    for (input, mut character) in &mut q_player {
        apply_character_action(&time, /*&spatial_query,*/ input, &mut character);
    }
}

#[derive(Bundle)]
struct CharacterPhysicsBundle {
    collider: Collider,
    rigid_body: RigidBody,
    external_force: ExternalForce,
    external_impulse: ExternalImpulse,
    lock_axes: LockedAxes,
    friction: Friction,
}

impl Default for CharacterPhysicsBundle {
    fn default() -> Self {
        Self {
            collider: Collider::capsule(CHARACTER_CAPSULE_RADIUS, CHARACTER_CAPSULE_HEIGHT),
            rigid_body: RigidBody::Dynamic,
            external_force: ExternalForce::ZERO.with_persistence(false),
            external_impulse: ExternalImpulse::ZERO.with_persistence(false),
            lock_axes: LockedAxes::default()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z(),
            friction: Friction::new(0.0).with_combine_rule(CoefficientCombine::Min),
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable, derive(Debug))]
struct CharacterQuery {
    external_force: &'static mut ExternalForce,
    external_impulse: &'static mut ExternalImpulse,
    linear_velocity: &'static LinearVelocity,
    mass: &'static ComputedMass,
    position: &'static Position,
    rotation: &'static mut Rotation,
    entity: Entity,
}

/// Apply the character actions `action_state` to the character entity `character`.
fn apply_character_action(
    time: &Res<Time>,
    // spatial_query: &SpatialQuery,
    input: &PlayerCharacterInput,
    character: &mut CharacterQueryItem,
) {
    // TODO: unhardcode these values
    const MAX_SPEED: f32 = 5.0;
    const MAX_ACCELERATION: f32 = 20.0;
    const LOOK_SENSITIVITY: f32 = 0.0025;

    // === ROTATION ===
    // Rotate player around Y axis by look.x (yaw)
    let yaw_delta = -input.look.x * LOOK_SENSITIVITY;
    let yaw_rotation = Quat::from_rotation_y(yaw_delta);
    character.rotation.0 = yaw_rotation * character.rotation.0;

    // === MOVEMENT ===
    let move_input = input.move_axis.clamp_length_max(1.0);
    let local_move = Vec3::new(move_input.x, 0.0, -move_input.y); // X=strafe, Y=forward

    // Rotate move direction by the player's current facing rotation
    let world_move = character.rotation.0 * local_move;

    // Current horizontal velocity
    let ground_linear_velocity = Vec3::new(
        character.linear_velocity.x,
        0.0,
        character.linear_velocity.z,
    );

    // Desired velocity
    let desired_ground_linear_velocity = world_move * MAX_SPEED;

    // Smooth acceleration
    let max_velocity_delta_per_tick = MAX_ACCELERATION * time.delta_secs();
    let new_ground_linear_velocity = ground_linear_velocity
        .move_towards(desired_ground_linear_velocity, max_velocity_delta_per_tick);

    let required_acceleration =
        (new_ground_linear_velocity - ground_linear_velocity) / time.delta_secs();

    character
        .external_force
        .apply_force(required_acceleration * character.mass.value());

    // // Handle jumping.
    // if input.jump {
    //     let ray_cast_origin = character.position.0
    //         + Vec3::new(
    //             0.0,
    //             -CHARACTER_CAPSULE_HEIGHT / 2.0 - CHARACTER_CAPSULE_RADIUS,
    //             0.0,
    //         );

    //     // Only jump if the character is on the ground.
    //     //
    //     // Check if we are touching the ground by sending a ray from the bottom
    //     // of the character downwards.
    //     if spatial_query
    //         .cast_ray(
    //             ray_cast_origin,
    //             Dir3::NEG_Y,
    //             0.01,
    //             true,
    //             &SpatialQueryFilter::from_excluded_entities([character.entity]),
    //         )
    //         .is_some()
    //     {
    //         character
    //             .external_impulse
    //             .apply_impulse(Vec3::new(0.0, 5.0, 0.0));
    //     }
    // }
}
