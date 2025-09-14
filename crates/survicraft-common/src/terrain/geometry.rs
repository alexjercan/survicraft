use bevy::{
    asset::RenderAssetUsages,
    platform::collections::HashMap,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use itertools::Itertools;

use crate::{terrain::prelude::*, tilemap::prelude::*};

// #[cfg(feature = "debug")]
// use self::debug::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TerrainGeometryPluginSet;

pub struct TerrainGeometryPlugin {
    tile_size: Vec2,
    max_height: f64,
}

impl TerrainGeometryPlugin {
    pub fn new(tile_size: Vec2, max_height: f64) -> Self {
        Self {
            tile_size,
            max_height,
        }
    }
}

impl Plugin for TerrainGeometryPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GeometrySettings>();

        // #[cfg(feature = "debug")]
        // app.add_plugins(DebugPlugin);
        // #[cfg(feature = "debug")]
        // app.configure_sets(Update, DebugPluginSet.in_set(TerrainRenderPluginSet));

        app.insert_resource(GeometrySettings::new(
            self.tile_size,
            self.max_height,
        ))
        .add_systems(Update, generate_chunk_mesh.in_set(TerrainGeometryPluginSet));
    }
}

#[derive(Component)]
struct ChunkMeshReady;

#[derive(Resource, Debug, Clone, Default, Reflect)]
struct GeometrySettings {
    tile_size: Vec2,
    max_height: f64,
}

impl GeometrySettings {
    fn new(tile_size: Vec2, max_height: f64) -> Self {
        Self {
            tile_size,
            max_height,
        }
    }

    fn mesh(&self, chunk: HashMap<IVec2, f64>) -> Mesh {
        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();

        let tile_size = self.tile_size;
        let half_size = tile_size * 0.5;

        for (tile_coord, _) in chunk.iter() {
            let base_x = tile_coord.x as f32 * tile_size.x;
            let base_y = tile_coord.y as f32 * tile_size.y;

            let base = Vec2::new(base_x, base_y);

            let v0 = base + Vec2::new(-half_size.x, -half_size.y);
            let v1 = base + Vec2::new(half_size.x, -half_size.y);
            let v2 = base + Vec2::new(-half_size.x, half_size.y);
            let v3 = base + Vec2::new(half_size.x, half_size.y);

            let ns = vec![
                IVec2::new(0, 0),
                IVec2::new(-1, -1),
                IVec2::new(0, -1),
                IVec2::new(-1, 0),
            ]
            .iter()
            .filter_map(|offset| chunk.get(&(tile_coord + *offset)).copied())
            .collect_vec();
            if ns.len() < 4 {
                continue;
            }
            let h = if ns.iter().any(|n| *n <= 0.0) {
                0.0
            } else {
                ns.iter().sum::<f64>() / 4.0
            };
            let v0 = Vec3::new(v0.x, h as f32, v0.y);

            let ns = vec![
                IVec2::new(0, 0),
                IVec2::new(1, -1),
                IVec2::new(0, -1),
                IVec2::new(1, 0),
            ]
            .iter()
            .filter_map(|offset| chunk.get(&(tile_coord + *offset)).copied())
            .collect_vec();
            if ns.len() < 4 {
                continue;
            }
            let h = if ns.iter().any(|n| *n <= 0.0) {
                0.0
            } else {
                ns.iter().sum::<f64>() / 4.0
            };
            let v1 = Vec3::new(v1.x, h as f32, v1.y);

            let ns = vec![
                IVec2::new(0, 0),
                IVec2::new(-1, 1),
                IVec2::new(0, 1),
                IVec2::new(-1, 0),
            ]
            .iter()
            .filter_map(|offset| chunk.get(&(tile_coord + *offset)).copied())
            .collect_vec();
            if ns.len() < 4 {
                continue;
            }
            let h = if ns.iter().any(|n| *n <= 0.0) {
                0.0
            } else {
                ns.iter().sum::<f64>() / 4.0
            };
            let v2 = Vec3::new(v2.x, h as f32, v2.y);

            let ns = vec![
                IVec2::new(0, 0),
                IVec2::new(1, 1),
                IVec2::new(0, 1),
                IVec2::new(1, 0),
            ]
            .iter()
            .filter_map(|offset| chunk.get(&(tile_coord + *offset)).copied())
            .collect_vec();
            if ns.len() < 4 {
                continue;
            }
            let h = if ns.iter().any(|n| *n <= 0.0) {
                0.0
            } else {
                ns.iter().sum::<f64>() / 4.0
            };
            let v3 = Vec3::new(v3.x, h as f32, v3.y);

            let start_index = positions.len() as u32;

            positions.push(v0);
            positions.push(v1);
            positions.push(v2);
            positions.push(v3);

            // UVs
            uvs.push([0.0, 0.0]);
            uvs.push([1.0, 0.0]);
            uvs.push([0.0, 1.0]);
            uvs.push([1.0, 1.0]);

            // Indices for two triangles
            indices.push(start_index);
            indices.push(start_index + 2);
            indices.push(start_index + 1);

            indices.push(start_index + 1);
            indices.push(start_index + 2);
            indices.push(start_index + 3);
        }

        let mut normals = vec![Vec3::ZERO; positions.len()];

        for tri in indices.chunks(3) {
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;

            let v0 = positions[i0];
            let v1 = positions[i1];
            let v2 = positions[i2];

            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let n = edge1.cross(edge2).normalize();

            normals[i0] += n;
            normals[i1] += n;
            normals[i2] += n;
        }

        // normalize all
        for n in &mut normals {
            *n = n.normalize();
        }

        // Create mesh
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
    }
}

fn generate_chunk_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    layout: Res<GeometrySettings>,
    q_tiles: Query<(Entity, &LocalTileCoord, &TileNoiseHeight, &ChildOf), Without<ChunkMeshReady>>,
) {
    if q_tiles.is_empty() {
        return;
    }
    debug!("Handling chunk mesh for {} tiles", q_tiles.iter().len());

    for (chunk_entity, chunk) in q_tiles
        .iter()
        .sorted_by_key(|(_, _, _, ChildOf(e))| *e)
        .chunk_by(|(_, _, _, ChildOf(e))| *e)
        .into_iter()
    {
        let mut storage = HashMap::default();

        for (entity, local, height, _) in chunk {
            let height = **height;
            let height_value = (height * 2.0 - 1.0).clamp(0.0, 1.0); // flatten 0.5 and below to 0.0 hardcoded for water
            let height_mesh = (height_value * layout.max_height).round();

            storage.insert(**local, height_mesh);

            commands.entity(entity).insert(ChunkMeshReady);
        }

        let mesh = layout.mesh(storage);

        commands.entity(chunk_entity).with_children(|parent| {
            parent.spawn((
                Name::new("Chunk Mesh"),
                ChunkMesh,
                Visibility::Visible,
                Mesh3d(meshes.add(mesh.clone())),
            ));
        });
    }
}
