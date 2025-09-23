use crate::{common::prelude::*, protocol::prelude::*};
use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::input::{config::InputConfig, leafwing::prelude::*};
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

/// Marker component for the player character entity. Spawn this when you
/// want to attach a player bundle and have it be controlled by a player.
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct NetworkPlayerController;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
struct HeadControllerMarker;

pub struct NetworkPlayerControllerPlugin {
    pub render: bool,
}

impl Plugin for NetworkPlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<NetworkPlayerController>();

        // Register the player controller for networking
        app.register_component::<NetworkPlayerController>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<HeadControllerMarker>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.add_plugins(PhysicsCharacterPlugin);
        app.add_plugins(HeadControllerPlugin);

        // Add input handling for network rebroadcasting
        app.add_plugins(InputPlugin::<NetworkCharacterAction> {
            config: InputConfig::<NetworkCharacterAction> {
                rebroadcast_inputs: true,
                ..default()
            },
        });
        app.add_plugins(InputManagerPlugin::<NetworkHeadAction>::default());

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
            ),
        );
        app.add_systems(FixedUpdate, sync_character_rotation);
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
pub enum NetworkCharacterAction {
    #[actionlike(DualAxis)]
    Move,
    Jump,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
pub enum NetworkHeadAction {
    #[actionlike(DualAxis)]
    Look,
}

fn on_add_player_controller(
    mut commands: Commands,
    q_player: Query<
        (Entity, &PlayerId, Has<Controlled>),
        (Added<Predicted>, With<NetworkPlayerController>),
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
                    .with_dual_axis(NetworkHeadAction::Look, GamepadStick::RIGHT)
                    .with_dual_axis(NetworkHeadAction::Look, MouseMove::default()),
                Camera3d::default(), // NOTE: Careful when self.render = false
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
                .with(NetworkCharacterAction::Jump, KeyCode::Space)
                .with(NetworkCharacterAction::Jump, GamepadButton::South)
                .with_dual_axis(NetworkCharacterAction::Move, GamepadStick::LEFT)
                .with_dual_axis(NetworkCharacterAction::Move, VirtualDPad::wasd()),));
        } else {
            debug!("Remote character predicted for us: {entity:?}");
        }
    }
}

fn add_head_controller_to_new_players(
    mut commands: Commands,
    q_head: Query<(Entity, &PlayerId), (With<HeadControllerMarker>, Without<HeadControllerTarget>)>,
    q_player: Query<(Entity, &PlayerId), With<NetworkPlayerController>>,
) {
    for (entity, PlayerId(peer)) in &q_head {
        let player = match q_player.iter().find(|(_, id)| id.0 == *peer) {
            Some((e, _)) => e,
            None => {
                error!("No player entity found for HeadControllerMarker with PlayerId {peer:?}");
                continue;
            }
        };

        trace!("Linking head controller {entity:?} to player entity {player:?}");
        commands.entity(entity).insert(HeadControllerTarget(player));
    }
}

fn update_character_input(
    mut q_player: Query<(&mut PhysicsCharacterInput, &ActionState<NetworkCharacterAction>)>,
) {
    for (mut input, action_state) in q_player.iter_mut() {
        input.move_axis = action_state.axis_pair(&NetworkCharacterAction::Move);
        input.jump = action_state.just_pressed(&NetworkCharacterAction::Jump);
    }
}

fn update_head_input(mut q_head: Query<(&mut HeadControllerInput, &ActionState<NetworkHeadAction>)>) {
    for (mut input, action_state) in q_head.iter_mut() {
        input.look_axis = action_state.axis_pair(&NetworkHeadAction::Look);
    }
}

fn sync_character_rotation(
    mut q_player: Query<&mut Rotation, With<NetworkPlayerController>>,
    q_head: Query<(&Rotation, &HeadControllerTarget), Without<NetworkPlayerController>>,
) {
    for (rotation, &HeadControllerTarget(target)) in q_head.iter() {
        let mut target_rotation = match q_player.get_mut(target) {
            Ok(r) => r,
            Err(_) => {
                // NOTE: This can happen when the client side does a rollback. Apparently lighyear
                // removes the component that is being rolled back, not sure why, but just ignore
                // it for now.
                // warn!("HeadControllerTarget entity {target:?} does not have a Rotation");
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
    q_player: Query<(Entity, Has<Controlled>), (Added<Predicted>, With<NetworkPlayerController>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, is_controlled) in q_player.iter() {
        if is_controlled {
            // NOTE: we don't render the local player character since the camera is inside it
            continue;
        }
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
