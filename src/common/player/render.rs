//! TODO: add description

use crate::common::player::prelude::*;
use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerRenderPluginSet;

pub struct PlayerRenderPlugin;

impl Plugin for PlayerRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_render_player.in_set(PlayerRenderPluginSet));
    }
}

fn handle_render_player(
    mut commands: Commands,
    q_player: Query<Entity, Added<PlayerCharacterController>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in &q_player {
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
