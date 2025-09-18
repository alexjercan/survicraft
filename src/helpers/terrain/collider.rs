//! TODO: Documentation

use crate::helpers::tilemap::prelude::*;
use super::components::*;
use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TerrainColliderPluginSet;

pub struct TerrainColliderPlugin;

impl Plugin for TerrainColliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (generate_chunk_collider).in_set(TerrainColliderPluginSet),
        );
    }
}

#[derive(Component)]
struct ChunkColliderReady;

fn generate_chunk_collider(
    mut commands: Commands,
    q_meshes: Query<(Entity, &ChunkMesh, &ChildOf), Without<ChunkColliderReady>>,
    q_chunks: Query<Entity, With<ChunkCoord>>,
) {
    if q_meshes.is_empty() {
        return;
    }
    debug!("Generating collider for {} meshes", q_meshes.iter().len());

    for (entity, ChunkMesh(mesh), ChildOf(parent)) in q_meshes.iter() {
        commands.entity(entity).insert((
            ChunkColliderReady,
            Collider::trimesh_from_mesh(mesh).unwrap(),
            RigidBody::Static,
        ));

        if let Some(parent) = q_chunks.get(*parent).ok() {
            commands.entity(parent).insert(ChunkReady);
        }
    }
}
