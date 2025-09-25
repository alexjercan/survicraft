//! TODO: Add documentation

use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderRef},
        storage::ShaderStorageBuffer,
    },
};

#[cfg(feature = "debug")]
use self::debug::*;
use super::{components::*, resources::*};
use crate::common::prelude::*;

pub struct TerrainRenderPlugin {
    tile_size: Vec2,
    chunk_radius: u32,
}

impl Default for TerrainRenderPlugin {
    fn default() -> Self {
        Self {
            tile_size: super::TILE_SIZE,
            chunk_radius: super::CHUNK_RADIUS,
        }
    }
}

impl TerrainRenderPlugin {
    pub fn new(tile_size: Vec2, chunk_radius: u32) -> Self {
        Self {
            tile_size,
            chunk_radius,
        }
    }
}

impl Plugin for TerrainRenderPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RenderSettings>();

        #[cfg(feature = "debug")]
        app.add_plugins(DebugPlugin);

        app.insert_resource(RenderSettings::new(self.tile_size, self.chunk_radius))
            .add_plugins(MaterialPlugin::<
                ExtendedMaterial<StandardMaterial, ChunkMaterial>,
            >::default())
            .add_systems(Update, generate_chunk_render);
    }
}

#[derive(Component)]
struct ChunkRenderReady;

#[derive(Resource, Debug, Clone, Default, Reflect)]
struct RenderSettings {
    tile_size: Vec2,
    chunk_radius: u32,
}

impl RenderSettings {
    fn new(tile_size: Vec2, chunk_radius: u32) -> Self {
        Self {
            tile_size,
            chunk_radius,
        }
    }
}

fn generate_chunk_render(
    mut commands: Commands,
    mut chunk_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ChunkMaterial>>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut meshes: ResMut<Assets<Mesh>>,
    layout: Res<RenderSettings>,
    q_tiles: Query<(&LocalTileCoord, &Tile), Without<TileBorder>>,
    q_meshes: Query<(Entity, &ChunkMesh, &ChildOf), Without<ChunkRenderReady>>,
    q_chunks: Query<&Children, With<ChunkCoord>>,
    assets: Res<TerrainAssets>,
) {
    if q_meshes.is_empty() {
        return;
    }
    trace!(
        "Generating render data for {} chunk meshes",
        q_meshes.iter().len()
    );

    let size = layout.chunk_radius * 2 + 1;
    for (entity, ChunkMesh(mesh), ChildOf(chunk)) in q_meshes.iter() {
        if let Ok(children) = q_chunks.get(*chunk) {
            let mut height_data = vec![-1; (size * size) as usize];

            for child in children.iter() {
                if let Ok((local, tile)) = q_tiles.get(child) {
                    let q_offset = local.x + layout.chunk_radius as i32;
                    let r_offset = local.y + layout.chunk_radius as i32;
                    let index = (r_offset * size as i32 + q_offset) as usize;
                    height_data[index] = assets.get_tile_index(tile).map_or(-1, |i| i as i32);
                }
            }

            commands.entity(entity).insert((
                ChunkRenderReady,
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
            ));
        }
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

#[cfg(feature = "debug")]
mod debug {
    use bevy::{
        pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
        render::mesh::VertexAttributeValues,
    };

    use super::*;

    #[derive(Debug, Resource, Default, Clone, Deref, DerefMut)]
    struct ShowGrid(pub bool);

    pub struct DebugPlugin;

    impl Plugin for DebugPlugin {
        fn build(&self, app: &mut App) {
            app.add_plugins(WireframePlugin::default())
                .insert_resource(WireframeConfig {
                    global: false,
                    default_color: Color::WHITE,
                });
            app.insert_resource(ShowGrid(true));
            app.add_systems(Update, (toggle, draw_grid, undraw_grid, draw_normals));
        }
    }

    fn toggle(kbd: Res<ButtonInput<KeyCode>>, mut show_grid: ResMut<ShowGrid>) {
        if kbd.just_pressed(KeyCode::F11) {
            show_grid.0 = !show_grid.0;
        }
    }

    fn draw_grid(
        mut commands: Commands,
        show_grid: Res<ShowGrid>,
        q_meshes: Query<(Entity, &ChildOf, Has<Wireframe>), With<ChunkMesh>>,
        q_chunks: Query<&ChunkCoord>,
        q_camera: Query<&Transform, With<Camera3d>>,
        storage: Res<TileMapStorage>,
    ) {
        if !**show_grid {
            return;
        }

        let Ok(transform) = q_camera.single() else {
            return;
        };

        let tile = storage.world_pos_to_tile(transform.translation.xz());
        let center = storage.tile_to_center(&tile);

        for (entity, ChildOf(parent), has_wireframe) in q_meshes.iter() {
            let Ok(ChunkCoord(coord)) = q_chunks.get(*parent) else {
                continue;
            };

            if center == *coord {
                if has_wireframe {
                    continue;
                }
                commands.entity(entity).insert(Wireframe);
            } else {
                commands.entity(entity).remove::<Wireframe>();
            }
        }
    }

    fn undraw_grid(
        mut commands: Commands,
        show_grid: Res<ShowGrid>,
        q_meshes: Query<Entity, (With<ChunkMesh>, With<Wireframe>)>,
    ) {
        if **show_grid {
            return;
        }

        for entity in q_meshes.iter() {
            commands.entity(entity).remove::<Wireframe>();
        }
    }

    fn draw_normals(
        mut gizmos: Gizmos,
        show_grid: Res<ShowGrid>,
        q_meshes: Query<(&ChunkMesh, &ChildOf, &GlobalTransform)>,
        q_chunks: Query<&ChunkCoord>,
        q_camera: Query<&Transform, With<Camera3d>>,
        storage: Res<TileMapStorage>,
    ) {
        if !**show_grid {
            return;
        }

        let Ok(transform) = q_camera.single() else {
            return;
        };

        let tile = storage.world_pos_to_tile(transform.translation.xz());
        let center = storage.tile_to_center(&tile);

        let line_length = 0.25;
        for (mesh, ChildOf(parent), transform) in &q_meshes {
            let Ok(ChunkCoord(coord)) = q_chunks.get(*parent) else {
                continue;
            };

            if center != *coord {
                continue;
            }

            let Some(VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                continue;
            };

            let Some(VertexAttributeValues::Float32x3(normals)) =
                mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
            else {
                continue;
            };

            for (pos, normal) in positions.iter().zip(normals.iter()) {
                let pos = transform.transform_point(Vec3::from_array(*pos));
                let normal = transform.rotation() * Vec3::from_array(*normal);

                gizmos.line(
                    pos,
                    pos + normal.normalize() * line_length,
                    Color::srgb(0.0, 1.0, 0.0),
                );
            }
        }
    }
}
