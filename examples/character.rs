use avian3d::prelude::*;
use bevy::{ecs::query::QueryData, prelude::*};
use clap::Parser;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};
use survicraft::prelude::new_gui_app;

#[derive(Parser)]
#[command(name = "survicraft-character")]
#[command(version = "0.1")]
#[command(about = "Example for the survicraft character controller", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();

    let mut app = new_gui_app();

    // Setup the physics plugins for the character controller example
    app.add_plugins(
        PhysicsPlugins::default()
            .build()
    );

    app.add_systems(Startup, setup);
    app.add_plugins(PlayerControllerPlugin { render: true });

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(60.0, 60.0, 00.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Directional Light"),
    ));

    commands.spawn((
        PlayerController,
        Transform::from_xyz(0.0, 3.0, 0.0),
        Name::new("Player Character"),
    ));

    const FLOOR_WIDTH: f32 = 10.0;
    const FLOOR_HEIGHT: f32 = 1.0;
    commands.spawn((
        Collider::cuboid(FLOOR_WIDTH, FLOOR_HEIGHT, FLOOR_WIDTH),
        RigidBody::Static,
        Position::new(Vec3::ZERO),
        Mesh3d(meshes.add(Cuboid::new(FLOOR_WIDTH, FLOOR_HEIGHT, FLOOR_WIDTH))),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
    ));
}

struct PlayerControllerPlugin {
    render: bool,
}

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsCharacterPlugin);
        app.add_plugins(HeadControllerPlugin);

        app.add_plugins(InputManagerPlugin::<CharacterAction>::default());
        app.add_plugins(InputManagerPlugin::<HeadAction>::default());

        if self.render {
            app.add_plugins(PlayerRenderPlugin);
        }

        app.add_observer(on_add_player_controller);

        app.add_systems(Update, (update_character_input, update_head_input, sync_character_rotation));
    }
}

/// Marker component for the player character entity. Spawn this when you
/// want to attach a player bundle and have it be controlled by a player.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerController;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
pub enum CharacterAction {
    #[actionlike(DualAxis)]
    Move,
    Jump,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
pub enum HeadAction {
    #[actionlike(DualAxis)]
    Look,
}

fn on_add_player_controller(trigger: Trigger<OnAdd, PlayerController>, mut commands: Commands) {
    let entity = trigger.target();

    commands.spawn((
        InputMap::default()
            .with_dual_axis(HeadAction::Look, GamepadStick::RIGHT)
            .with_dual_axis(HeadAction::Look, MouseMove::default()),
        HeadController {
            offset: Vec3::new(0.0, CHARACTER_CAPSULE_HEIGHT / 2.0, 0.0),
            ..default()
        },
        HeadControllerInput::default(),
        HeadControllerTarget(entity),
        Camera3d::default(),
    ));

    commands.entity(entity).insert((
        InputMap::new([(CharacterAction::Jump, KeyCode::Space)])
            .with(CharacterAction::Jump, GamepadButton::South)
            .with_dual_axis(CharacterAction::Move, GamepadStick::LEFT)
            .with_dual_axis(CharacterAction::Move, VirtualDPad::wasd()),
        PhysicsCharacterBundle::default(),
        PhysicsCharacterInput::default(),
    ));
}

fn update_character_input(
    mut q_player: Query<(&mut PhysicsCharacterInput, &ActionState<CharacterAction>)>,
) {
    for (mut input, action_state) in q_player.iter_mut() {
        input.move_axis = action_state.axis_pair(&CharacterAction::Move);
        input.jump = action_state.just_pressed(&CharacterAction::Jump);
    }
}

fn update_head_input(mut q_head: Query<(&mut HeadControllerInput, &ActionState<HeadAction>)>) {
    for (mut input, action_state) in q_head.iter_mut() {
        input.look_axis = action_state.axis_pair(&HeadAction::Look);
    }
}

fn sync_character_rotation(
    mut q_player: Query<&mut Rotation, With<PlayerController>>,
    q_head: Query<(&Transform, &HeadControllerTarget), With<HeadController>>,
) {
    for (transform, &HeadControllerTarget(target)) in q_head.iter() {
        let mut rotation = match q_player.get_mut(target) {
            Ok(r) => r,
            Err(_) => {
                warn!("HeadControllerTarget entity {target:?} does not have a Rotation");
                continue;
            }
        };

        let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
        rotation.0 = Quat::from_euler(EulerRot::YXZ, yaw, 0.0, 0.0);
    }
}

// --- Head Camera Plugin ---

/// The Head camera component, which allows for mouse look.
#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct HeadController {
    /// The look sensitivity of the camera
    pub look_sensitivity: f32,
    /// The offset of the camera from the object it is following
    pub offset: Vec3,
    /// The minimum pitch of the camera
    pub min_pitch: f32,
    /// The maximum pitch of the camera
    pub max_pitch: f32,
}

impl Default for HeadController {
    fn default() -> Self {
        Self {
            look_sensitivity: 0.0025,
            offset: Vec3::new(0.0, 1.5, 0.0),
            min_pitch: std::f32::consts::FRAC_PI_2 * -0.99,
            max_pitch: std::f32::consts::FRAC_PI_2 * 0.99,
        }
    }
}

/// The input component for the Head camera, which stores the current input state.
/// This component should be updated by user input systems to control the camera.
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
pub struct HeadControllerInput {
    pub look_axis: Vec2,
}

/// Target for the Head camera to follow.
#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct HeadControllerTarget(pub Entity);

pub struct HeadControllerPlugin;

impl Plugin for HeadControllerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HeadController>()
            .register_type::<HeadControllerInput>()
            .register_type::<HeadControllerTarget>();

        app.add_systems(Update, sync_transform);
    }
}

fn sync_transform(
    mut q_camera: Query<(
        &mut Transform,
        &HeadControllerInput,
        &HeadController,
        &HeadControllerTarget,
    )>,
    q_target: Query<&GlobalTransform, Without<HeadController>>,
) {
    for (mut transform, input, camera, &HeadControllerTarget(target)) in q_camera.iter_mut() {
        let target_transform = match q_target.get(target) {
            Ok(t) => t,
            Err(_) => {
                warn!("HeadControllerTarget entity {target:?} does not have a GlobalTransform");
                continue;
            }
        };

        let pitch_delta = -input.look_axis.y * camera.look_sensitivity;
        let (_, current_pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let new_pitch = (current_pitch + pitch_delta).clamp(camera.min_pitch, camera.max_pitch);

        let yaw_delta = -input.look_axis.x * camera.look_sensitivity;
        let (target_yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let new_yaw = target_yaw + yaw_delta;

        transform.rotation = Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, 0.0);

        transform.translation = target_transform.translation() + camera.offset;
    }
}

// --- Character Plugin ---

pub const CHARACTER_CAPSULE_RADIUS: f32 = 0.5;
pub const CHARACTER_CAPSULE_HEIGHT: f32 = 1.0;

#[derive(Bundle)]
pub struct PhysicsCharacterBundle {
    collider: Collider,
    rigid_body: RigidBody,
    external_force: ExternalForce,
    external_impulse: ExternalImpulse,
    lock_axes: LockedAxes,
    friction: Friction,
}

impl Default for PhysicsCharacterBundle {
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

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
pub struct PhysicsCharacterInput {
    pub move_axis: Vec2,
    pub jump: bool,
}

pub struct PhysicsCharacterPlugin;

impl Plugin for PhysicsCharacterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PhysicsCharacterInput>();

        app.add_systems(FixedUpdate, handle_character_actions);
    }
}

fn handle_character_actions(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut q_player: Query<(&PhysicsCharacterInput, CharacterQuery)>,
) {
    for (input, mut character) in &mut q_player {
        apply_character_action(&time, &spatial_query, input, &mut character);
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
    rotation: &'static Rotation,
    entity: Entity,
}

/// Apply the character actions `action_state` to the character entity `character`.
fn apply_character_action(
    time: &Res<Time>,
    spatial_query: &SpatialQuery,
    input: &PhysicsCharacterInput,
    character: &mut CharacterQueryItem,
) {
    // TODO: unhardcode these values
    const MAX_SPEED: f32 = 5.0;
    const MAX_ACCELERATION: f32 = 20.0;
    const JUMP_FORCE: f32 = 5.0;

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

    if required_acceleration.length_squared() >= f32::EPSILON {
        character
            .external_force
            .apply_force(required_acceleration * character.mass.value());
    }

    // Handle jumping.
    if input.jump {
        let ray_cast_origin = character.position.0
            + Vec3::new(
                0.0,
                -CHARACTER_CAPSULE_HEIGHT / 2.0 - CHARACTER_CAPSULE_RADIUS,
                0.0,
            );

        // Only jump if the character is on the ground.
        //
        // Check if we are touching the ground by sending a ray from the bottom
        // of the character downwards.
        if spatial_query
            .cast_ray(
                ray_cast_origin,
                Dir3::NEG_Y,
                0.1,
                true,
                &SpatialQueryFilter::from_excluded_entities([character.entity]),
            )
            .is_some()
        {
            character
                .external_impulse
                .apply_impulse(Vec3::new(0.0, JUMP_FORCE, 0.0));
        }
    }
}

pub struct PlayerRenderPlugin;

impl Plugin for PlayerRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_render_player);
    }
}

fn handle_render_player(
    q_player: Query<Entity, Added<PlayerController>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in q_player.iter() {
        debug!("Rendering player entity {entity:?}");

        commands.entity(entity).insert((
            Mesh3d(meshes.add(Mesh::from(Capsule3d {
                radius: CHARACTER_CAPSULE_RADIUS,
                half_length: CHARACTER_CAPSULE_HEIGHT / 2.0,
                ..default()
            }))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.7, 0.6),
                ..default()
            })),
        ));
    }
}
