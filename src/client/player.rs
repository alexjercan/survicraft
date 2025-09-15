//! The player plugin handles the client side player logic.

use crate::common::prelude::*;
use crate::protocol::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PlayerPluginSet;

pub(crate) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            handle_character_actions.in_set(PlayerPluginSet),
        );
        app.add_systems(Update, handle_spawn_player.in_set(PlayerPluginSet));
    }
}

fn handle_spawn_player(
    mut commands: Commands,
    mut q_player: Query<(Entity, Has<Controlled>), (Added<Predicted>, With<PlayerId>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, is_controlled) in &mut q_player {
        if is_controlled {
            debug!("Adding InputMap to controlled and predicted entity {entity:?}");
            commands.entity(entity).insert(
                InputMap::new([(CharacterAction::Jump, KeyCode::Space)])
                    .with(CharacterAction::Jump, GamepadButton::South)
                    .with_dual_axis(CharacterAction::Move, GamepadStick::LEFT)
                    .with_dual_axis(CharacterAction::Move, VirtualDPad::wasd()),
            );
        } else {
            debug!("Remote character predicted for us: {entity:?}");
        }

        debug!(?entity, "Adding physics to character");
        commands
            .entity(entity)
            .insert(CharacterPhysicsBundle::default())
            .insert((
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

/// Process character actions and apply them to their associated character
/// entity.
fn handle_character_actions(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut query: Query<(&ActionState<CharacterAction>, CharacterQuery), With<Predicted>>,
    // In host-server mode, the server portion is already applying the
    // character actions and so we don't want to apply the character
    // actions twice. This excludes host-server mode since there are multiple timelines
    // when running in host-server mode.
    timeline: Single<&LocalTimeline>,
) {
    let _tick = timeline.tick();
    for (action_state, mut character) in &mut query {
        // lightyear handles correctly both inputs from the local player or the remote player, during rollback
        // or out of rollback.
        // The ActionState is always updated to contain the correct action for the current tick.
        //
        // For remote players, we use the most recent input received
        apply_character_action(&time, &spatial_query, action_state, &mut character);
    }
}
