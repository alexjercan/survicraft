use avian3d::prelude::*;
use bevy::{ecs::query::QueryData, prelude::*};
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
