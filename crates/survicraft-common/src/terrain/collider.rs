use avian3d::prelude::*;
use bevy::prelude::*;

use crate::terrain::prelude::*;

// #[cfg(feature = "debug")]
// use self::debug::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TerrainColliderPluginSet;

pub struct TerrainColliderPlugin;

impl Plugin for TerrainColliderPlugin {
    fn build(&self, app: &mut App) {
        // #[cfg(feature = "debug")]
        // app.add_plugins(DebugPlugin);
        // #[cfg(feature = "debug")]
        // app.configure_sets(Update, DebugPluginSet.in_set(TerrainRenderPluginSet));

        app.add_systems(
            Update,
            (generate_chunk_collider).in_set(TerrainRenderPluginSet),
        );
    }
}

#[derive(Component)]
struct ChunkColliderReady;

fn generate_chunk_collider(
    mut commands: Commands,
    q_meshes: Query<(Entity, &ChunkMesh), (With<ChunkMesh>, Without<ChunkColliderReady>)>,
) {
    if q_meshes.is_empty() {
        return;
    }
    debug!("Generating collider for {} meshes", q_meshes.iter().len());

    for (entity, ChunkMesh(mesh)) in q_meshes.iter() {
        commands.entity(entity).insert((
            ChunkColliderReady,
            Collider::trimesh_from_mesh(mesh).unwrap(),
        ));
    }
}
