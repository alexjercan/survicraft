use crate::{helpers::prelude::*, protocol::prelude::*};
use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::input::{config::InputConfig, leafwing::prelude::*};
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

/// Marker component for the player character entity. Spawn this when you
/// want to attach a player bundle and have it be controlled by a player.
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct PlayerController;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
struct HeadControllerMarker;

pub struct PlayerControllerPlugin {
    pub render: bool,
}

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerController>();

        // Register the player controller for networking
        app.register_component::<PlayerController>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<HeadControllerMarker>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.add_plugins(PhysicsCharacterPlugin);
        app.add_plugins(HeadControllerPlugin);

        // Add input handling for network rebroadcasting
        app.add_plugins(InputPlugin::<CharacterAction> {
            config: InputConfig::<CharacterAction> {
                rebroadcast_inputs: true,
                ..default()
            },
        });
        app.add_plugins(InputManagerPlugin::<HeadAction>::default());

        if self.render {
            app.add_plugins(PlayerRenderPlugin);
        }

        app.add_systems(
            Update,
            (
                on_add_player_controller,
                add_head_controller_to_new_players,
                update_character_input,
                update_head_input,
                sync_character_rotation,
            ),
        );
    }
}

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

fn on_add_player_controller(
    mut commands: Commands,
    q_player: Query<
        (Entity, &PlayerId, Has<Controlled>),
        (Added<Predicted>, With<PlayerController>),
    >,
) {
    for (entity, PlayerId(peer), is_controlled) in &q_player {
        debug!("Adding PhysicsCharacterBundle to entity {entity:?}");

        commands.entity(entity).insert((
            PhysicsCharacterBundle::default(),
            PhysicsCharacterInput::default(),
        ));

        if is_controlled {
            debug!("Adding InputMap and Camera to controlled and predicted entity {entity:?}");

            commands.spawn((
                Name::new("Head"),
                InputMap::default()
                    .with_dual_axis(HeadAction::Look, GamepadStick::RIGHT)
                    .with_dual_axis(HeadAction::Look, MouseMove::default()),
                Camera3d::default(),
                HeadController {
                    offset: Vec3::new(0.0, CHARACTER_CAPSULE_HEIGHT / 2.0, 0.0),
                    ..default()
                },
                HeadControllerInput::default(),
                Transform::default(),
                Rotation::default(),
                PlayerId(*peer),
                HeadControllerMarker,
                HeadControllerTarget(entity),
                Replicate::to_server(),
            ));

            commands.entity(entity).insert((InputMap::default()
                .with(CharacterAction::Jump, KeyCode::Space)
                .with(CharacterAction::Jump, GamepadButton::South)
                .with_dual_axis(CharacterAction::Move, GamepadStick::LEFT)
                .with_dual_axis(CharacterAction::Move, VirtualDPad::wasd()),));
        } else {
            debug!("Remote character predicted for us: {entity:?}");
        }
    }
}

fn add_head_controller_to_new_players(
    mut commands: Commands,
    q_head: Query<(Entity, &PlayerId), (With<HeadControllerMarker>, Without<HeadControllerTarget>)>,
    q_player: Query<(Entity, &PlayerId), With<PlayerController>>,
) {
    for (entity, PlayerId(peer)) in &q_head {
        let player = match q_player.iter().find(|(_, id)| id.0 == *peer) {
            Some((e, _)) => e,
            None => {
                warn!("No player entity found for HeadControllerMarker with PlayerId {peer:?}");
                continue;
            }
        };

        debug!("Linking head controller {entity:?} to player entity {player:?}");
        commands.entity(entity).insert(HeadControllerTarget(player));
    }
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
    mut q_player: Query<(Entity, &mut Rotation), With<PlayerController>>,
    q_head: Query<(&Rotation, &HeadControllerTarget), Without<PlayerController>>
) {
    for (rotation, &HeadControllerTarget(target)) in q_head.iter() {
        let (_, mut target_rotation) = match q_player.get_mut(target) {
            Ok(r) => r,
            Err(_) => {
                warn!("HeadControllerTarget entity {target:?} does not have a Rotation");
                continue;
            }
        };

        let (yaw, _, _) = rotation.0.to_euler(EulerRot::YXZ);
        target_rotation.0 = Quat::from_euler(EulerRot::YXZ, yaw, 0.0, 0.0);
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
