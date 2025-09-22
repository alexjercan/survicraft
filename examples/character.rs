use avian3d::prelude::*;
use bevy::prelude::*;
use clap::Parser;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};
use survicraft::prelude::*;

#[derive(Parser)]
#[command(name = "survicraft-character")]
#[command(version = "0.1")]
#[command(about = "Example for the survicraft character controller", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();

    let mut app = new_gui_app();

    app.add_plugins(PhysicsPlugins::default().build());

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
        Position(Vec3::new(0.0, 3.0, 0.0)),
        Rotation::default(),
        PhysicsCharacterBundle::default(),
        PhysicsCharacterInput::default(),
    ));

    const FLOOR_WIDTH: f32 = 20.0;
    const FLOOR_HEIGHT: f32 = 1.0;

    commands.spawn((
        Name::new("Floor"),
        Collider::cuboid(FLOOR_WIDTH, FLOOR_HEIGHT, FLOOR_WIDTH),
        RigidBody::Static,
        Position::new(Vec3::ZERO),
        Mesh3d(meshes.add(Cuboid::new(FLOOR_WIDTH, FLOOR_HEIGHT, FLOOR_WIDTH))),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
    ));

    commands.spawn((
        Name::new("Ramp"),
        Collider::cuboid(FLOOR_WIDTH, FLOOR_HEIGHT, FLOOR_WIDTH),
        RigidBody::Static,
        Position::new(Vec3::new(
            -5.0,
            FLOOR_HEIGHT * std::f32::consts::FRAC_1_SQRT_2,
            0.0,
        )),
        Rotation::from(Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            0.0,
            0.0,
            -std::f32::consts::FRAC_PI_4,
        ))),
        Mesh3d(meshes.add(Cuboid::new(
            FLOOR_WIDTH,
            FLOOR_HEIGHT,
            FLOOR_WIDTH,
        ))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.8, 0.8))),
    ));

    for i in 0..5 {
        commands.spawn((
            Name::new(format!("Cube {i}")),
            Collider::cuboid(0.5, 0.5, 0.5),
            RigidBody::Dynamic,
            Position::new(Vec3::new(i as f32 - 2.0, 5.0 + i as f32, 0.0)),
            Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
        ));
    }

    for i in 0..5 {
        commands.spawn((
            Name::new(format!("Sphere {i}")),
            Collider::sphere(0.5),
            RigidBody::Dynamic,
            Position::new(Vec3::new(i as f32 - 2.0, 5.0 + i as f32, 2.0)),
            Mesh3d(meshes.add(Sphere::new(0.5))),
            MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.8))),
        ));
    }

    let wall_thickness = 1.0;
    let wall_height = 10.0;
    let wall_length = FLOOR_WIDTH * 2.0 + wall_thickness * 2.0;
    let wall_color = Color::srgb(0.4, 0.4, 0.4);
    let wall_material = materials.add(StandardMaterial {
        base_color: wall_color,
        ..default()
    });
    commands.spawn((
        Name::new("Wall +X"),
        Collider::cuboid(wall_thickness, wall_height, wall_length),
        RigidBody::Static,
        Position::new(Vec3::new(
            FLOOR_WIDTH / 2.0 + wall_thickness,
            wall_height / 2.0,
            0.0,
        )),
        Mesh3d(meshes.add(Cuboid::new(wall_thickness, wall_height, wall_length))),
        MeshMaterial3d(wall_material.clone()),
    ));
    commands.spawn((
        Name::new("Wall -X"),
        Collider::cuboid(wall_thickness, wall_height, wall_length),
        RigidBody::Static,
        Position::new(Vec3::new(
            -FLOOR_WIDTH / 2.0 - wall_thickness,
            wall_height / 2.0,
            0.0,
        )),
        Mesh3d(meshes.add(Cuboid::new(wall_thickness, wall_height, wall_length))),
        MeshMaterial3d(wall_material.clone()),
    ));
    commands.spawn((
        Name::new("Wall +Z"),
        Collider::cuboid(wall_length, wall_height, wall_thickness),
        RigidBody::Static,
        Position::new(Vec3::new(
            0.0,
            wall_height / 2.0,
            FLOOR_WIDTH / 2.0 + wall_thickness,
        )),
        Mesh3d(meshes.add(Cuboid::new(wall_length, wall_height, wall_thickness))),
        MeshMaterial3d(wall_material.clone()),
    ));
    commands.spawn((
        Name::new("Wall -Z"),
        Collider::cuboid(wall_length, wall_height, wall_thickness),
        RigidBody::Static,
        Position::new(Vec3::new(
            0.0,
            wall_height / 2.0,
            -FLOOR_WIDTH / 2.0 - wall_thickness,
        )),
        Mesh3d(meshes.add(Cuboid::new(wall_length, wall_height, wall_thickness))),
        MeshMaterial3d(wall_material),
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

        app.add_systems(
            Update,
            (
                update_character_input,
                update_head_input,
                sync_character_rotation,
            ),
        );
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
        Name::new("Head"),
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
        Transform::default(),
        Rotation::default(),
    ));

    commands
        .entity(entity)
        .insert((InputMap::new([(CharacterAction::Jump, KeyCode::Space)])
            .with(CharacterAction::Jump, GamepadButton::South)
            .with_dual_axis(CharacterAction::Move, GamepadStick::LEFT)
            .with_dual_axis(CharacterAction::Move, VirtualDPad::wasd()),));
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

struct PlayerRenderPlugin;

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
