//! The player plugin handles the client side player logic.

use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::{input::{leafwing::LeafwingSnapshot, InputBuffer}, *};
use crate::helpers::prelude::*;
use crate::protocol::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PlayerPluginSet;

pub(crate) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_spawn_player, handle_character_actions).in_set(PlayerPluginSet));
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
            info!("Adding InputMap to controlled and predicted entity {entity:?}");
            commands.entity(entity).insert(
                InputMap::new([(CharacterAction::Jump, KeyCode::Space)])
                    .with_dual_axis(CharacterAction::Move, GamepadStick::LEFT)
                    .with_dual_axis(CharacterAction::Move, VirtualDPad::wasd()),
            );
        } else {
            info!("Remote character predicted for us: {entity:?}");
        }

        info!(?entity, "Adding physics to character");
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

fn handle_character_actions(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut query: Query<
        (
            &ActionState<CharacterAction>,
            &InputBuffer<LeafwingSnapshot<CharacterAction>>,
            CharacterQuery,
        ),
        With<Predicted>,
    >,
    // In host-server mode, the server portion is already applying the
    // character actions and so we don't want to apply the character
    // actions twice. This excludes host-server mode since there are multiple timelines
    // when running in host-server mode.
    timeline: Single<&LocalTimeline>,
) {
    let tick = timeline.tick();
    for (action_state, input_buffer, mut character) in &mut query {
        // Use the current character action if it is present.
        if input_buffer.get(tick).is_some() {
            apply_character_action(&time, &spatial_query, action_state, &mut character);
            continue;
        }

        // If the current character action is not present then use the last real
        // character action.
        if let Some((_, prev_action_state)) = input_buffer.get_last_with_tick() {
            apply_character_action(&time, &spatial_query, prev_action_state, &mut character);
        } else {
            // No inputs are in the buffer yet. This can happen during initial
            // connection. Apply the default input (i.e. nothing pressed).
            apply_character_action(&time, &spatial_query, action_state, &mut character);
        }
    }
}
