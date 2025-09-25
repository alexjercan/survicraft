use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};
use lightyear::prelude::*;

use super::network::*;
use crate::prelude::*;

/// Marker component for the player character entity. Spawn this when you
/// want to attach a player bundle and have it be controlled by a player.
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub(super) struct PlayerController;

pub(super) struct PlayerControllerPlugin {
    pub render: bool,
}

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(KinematicCharacterPlugin);
        app.add_plugins(HeadControllerPlugin);

        app.add_plugins(NetworkInputPlugin::<CharacterAction>::default());
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
            ),
        );
        app.add_systems(FixedUpdate, (handle_spawn_player, sync_character_rotation));
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

fn handle_spawn_player(
    mut commands: Commands,
    mut ev_spawn: EventReader<ServerSpawnPlayerEvent>,
    q_player: Query<(Entity, &PlayerId), With<PlayerController>>,
) {
    for ServerSpawnPlayerEvent { owner, peer } in ev_spawn.read() {
        if q_player.iter().any(|(_, id)| id.0 == *peer) {
            warn!(
                "Player with ID {:?} already has a character, ignoring spawn request",
                peer
            );
            continue;
        }

        debug!("Spawning player character for peer {:?}", peer);

        commands.spawn((
            Name::new("Player"),
            PlayerController,
            ActionState::<CharacterAction>::default(),
            CharacterInput::default(),
            Position(Vec3::new(0.0, 3.0, 0.0)),
            Rotation::default(),
            KinematicCharacterBundle::default(),
            // Network related components
            PlayerId(*peer),
            Replicate::to_clients(NetworkTarget::All),
            PredictionTarget::to_clients(NetworkTarget::All),
            ControlledBy {
                owner: *owner,
                lifetime: Lifetime::default(),
            },
        ));
    }
}

fn on_add_player_controller(
    mut commands: Commands,
    q_player: Query<
        (Entity, &PlayerId, Has<Controlled>),
        (Added<Predicted>, With<PlayerController>),
    >,
) {
    for (entity, PlayerId(peer), is_controlled) in &q_player {
        debug!("Adding KinematicCharacterBundle to entity {entity:?}");

        commands.entity(entity).insert((
            KinematicCharacterBundle::default(),
            CharacterInput::default(),
        ));

        if is_controlled {
            debug!("Adding InputMap and Camera to controlled and predicted entity {entity:?}");

            commands.spawn((
                Name::new("Head"),
                InputMap::default()
                    .with_dual_axis(HeadAction::Look, GamepadStick::RIGHT)
                    .with_dual_axis(HeadAction::Look, MouseMove::default()),
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
            _ => {
                error!("No player entity found for HeadControllerMarker with PlayerId {peer:?}");
                continue;
            }
        };

        trace!("Linking head controller {entity:?} to player entity {player:?}");
        commands.entity(entity).insert(HeadControllerTarget(player));
    }
}

fn update_character_input(
    mut q_player: Query<(&mut CharacterInput, &ActionState<CharacterAction>)>,
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
    q_head: Query<(&Rotation, &HeadControllerTarget), Without<PlayerController>>,
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
    q_player: Query<(Entity, Has<Controlled>), (Added<Predicted>, With<PlayerController>)>,
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
