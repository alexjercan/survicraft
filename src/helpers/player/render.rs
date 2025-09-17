//! TODO: add description

use super::prelude::*;
use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerRenderPluginSet;

pub struct PlayerRenderPlugin;

impl Plugin for PlayerRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_render_player);
    }
}

fn handle_render_player(
    trigger: Trigger<OnAdd, PlayerCharacterController>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.entity(trigger.target()).insert((
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

// TODO: Add a debug plugin for this. It should show:
// - The player's velocity as an arrow
// - The player's grounded state
