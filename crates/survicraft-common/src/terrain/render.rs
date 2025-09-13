use bevy::{
    asset::RenderAssetUsages,
    pbr::{ExtendedMaterial, MaterialExtension},
    platform::collections::HashMap,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_resource::{AsBindGroup, ShaderRef},
        storage::ShaderStorageBuffer,
    },
};
use itertools::Itertools;

use crate::{terrain::prelude::*, tilemap::prelude::*};

// #[cfg(feature = "debug")]
// use self::debug::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TerrainRenderPluginSet;

pub struct TerrainRenderPlugin {
    tile_size: Vec2,
    chunk_radius: u32,
    max_height: f64,
}

impl TerrainRenderPlugin {
    pub fn new(tile_size: Vec2, chunk_radius: u32, max_height: f64) -> Self {
        Self {
            tile_size,
            chunk_radius,
            max_height,
        }
    }
}

impl Plugin for TerrainRenderPlugin {
    fn build(&self, app: &mut App) {
        // #[cfg(feature = "debug")]
        // app.add_plugins(DebugPlugin);
        // #[cfg(feature = "debug")]
        // app.configure_sets(Update, DebugPluginSet.in_set(TerrainRenderPluginSet));

        app.insert_resource(HeightMapLayout::new(
            self.tile_size,
            self.chunk_radius,
            self.max_height,
        ))
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, ChunkMaterial>,
        >::default())
        .add_systems(
            Update,
            (handle_render_height, handle_overlay_chunk).in_set(TerrainRenderPluginSet),
        );
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct HeightMapLayout {
    pub tile_size: Vec2,
    pub chunk_radius: u32,
    pub max_height: f64,
}

impl HeightMapLayout {
    fn new(tile_size: Vec2, chunk_radius: u32, max_height: f64) -> Self {
        Self {
            tile_size,
            chunk_radius,
            max_height,
        }
    }

    pub fn mesh(&self, chunk: HashMap<IVec2, f64>) -> Mesh {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();

        let tile_size = self.tile_size;
        let half_size = tile_size * 0.5;

        for (tile_coord, _) in chunk.iter() {
            let base_x = tile_coord.x as f32 * tile_size.x;
            let base_y = tile_coord.y as f32 * tile_size.y;

            let base = Vec2::new(base_x, base_y);

            // Vertices: bottom-left, bottom-right, top-left, top-right
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

            let h = ns.iter().sum::<f64>()
                / 4.0;
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
            let h = ns.iter().sum::<f64>()
                / 4.0;
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

            let h = ns.iter().sum::<f64>()
                / 4.0;
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

            let h = ns.iter().sum::<f64>()
                / 4.0;
            let v3 = Vec3::new(v3.x, h as f32, v3.y);

            let start_index = positions.len() as u32;

            positions.push(v0);
            positions.push(v1);
            positions.push(v2);
            positions.push(v3);

            // Normals (pointing up)
            normals.push(Vec3::Y);
            normals.push(Vec3::Y);
            normals.push(Vec3::Y);
            normals.push(Vec3::Y);

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

fn handle_render_height(
    mut commands: Commands,
    q_tiles: Query<(Entity, &TileNoiseHeight), Without<TileTopHeight>>,
    layout: Res<HeightMapLayout>,
) {
    if q_tiles.is_empty() {
        return;
    }
    debug!(
        "Handling tile top height for {} hexes",
        q_tiles.iter().len()
    );

    for (entity, height) in q_tiles.iter() {
        let height = **height;

        let height_value = (height * 2.0 - 1.0).clamp(0.0, 1.0); // flatten 0.5 and below to 0.0
        // (this is hardcoded for water)

        let height_mesh = (height_value * layout.max_height).round();

        commands.entity(entity).insert(TileTopHeight(height_mesh));
    }
}

#[derive(Component)]
struct ChunkMeshReady;

fn handle_overlay_chunk(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ChunkMaterial>>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    layout: Res<HeightMapLayout>,
    q_tiles: Query<
        (
            Entity,
            &LocalTileCoord,
            &TileTopHeight,
            &Tile,
            Option<&TileBorder>,
            &ChildOf,
        ),
        Without<ChunkMeshReady>,
    >,
    assets: Res<TerrainAssets>,
) {
    if q_tiles.is_empty() {
        return;
    }
    debug!("Handling chunk mesh for {} tiles", q_tiles.iter().len());

    let size = layout.chunk_radius * 2 + 1;
    for (chunk_entity, chunk) in q_tiles
        .iter()
        .sorted_by_key(|(_, _, _, _, _, ChildOf(e))| *e)
        .chunk_by(|(_, _, _, _, _, ChildOf(e))| *e)
        .into_iter()
    {
        let mut storage = HashMap::default();
        let mut height_data = vec![-1; (size * size) as usize];

        for (entity, local, height, tile, border, _) in chunk {
            commands.entity(entity).insert(ChunkMeshReady);
            storage.insert(**local, **height);

            if border.is_none() {
                let q_offset = local.x + layout.chunk_radius as i32;
                let r_offset = local.y + layout.chunk_radius as i32;
                let index = (r_offset * size as i32 + q_offset) as usize;
                height_data[index] = assets.get_tile_index(&**tile).map_or(-1, |i| i as i32);
            }
        }

        let mesh = layout.mesh(storage);

        commands.entity(chunk_entity).with_children(|parent| {
            parent.spawn((
                Visibility::Visible,
                Mesh3d(meshes.add(mesh.clone())),
                MeshMaterial3d(chunk_materials.add(ExtendedMaterial {
                    base: StandardMaterial {
                        perceptual_roughness: 1.0,
                        metallic: 0.0,
                        ..default()
                    },
                    extension: ChunkMaterial {
                        chunk_radius: layout.chunk_radius,
                        tile_size: layout.tile_size.x,
                        tiles: buffers.add(ShaderStorageBuffer::from(height_data)),
                    },
                })),
                Name::new("Chunk Mesh"),
            ));
        });
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ChunkMaterial {
    #[uniform(100)]
    pub chunk_radius: u32,
    #[uniform(101)]
    pub tile_size: f32,
    #[storage(102, read_only)]
    pub tiles: Handle<ShaderStorageBuffer>,
}

impl MaterialExtension for ChunkMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/tile_chunk.wgsl".into()
    }
}
