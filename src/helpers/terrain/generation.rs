//! TODO: Documentation

use super::{components::*, planet::*, resources::*};
use crate::helpers::{chunk_map::prelude::*, tilemap::prelude::*};
use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TerrainGenerationPluginSet;

pub struct TerrainGenerationPlugin {
    seed: u32,
    tile_size: Vec2,
    chunk_radius: u32,
    discover_radius: u32,
}

impl TerrainGenerationPlugin {
    pub fn new(seed: u32, tile_size: Vec2, chunk_radius: u32, discover_radius: u32) -> Self {
        Self {
            seed,
            tile_size,
            chunk_radius,
            discover_radius,
        }
    }
}

impl Plugin for TerrainGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Tile>()
            .register_type::<TileNoiseHeight>()
            .register_type::<TerrainGenerationProgress>();

        app.add_plugins(TileMapPlugin::new(
            self.tile_size,
            self.chunk_radius,
            self.discover_radius,
        ))
        .add_plugins(ChunkMapPlugin::<TileCoord, TileNoiseHeight, _>::new(
            PlanetHeight::default().with_seed(self.seed),
        ))
        .configure_sets(Update, TileMapSet.in_set(TerrainGenerationPluginSet))
        .configure_sets(Update, ChunkMapPluginSet.in_set(TerrainGenerationPluginSet))
        .add_systems(Update, handle_chunk.in_set(TerrainGenerationPluginSet));

        app.insert_resource(TerrainGenerationProgress::default());
        app.add_systems(
            Update,
            handle_chunk_progress.in_set(TerrainGenerationPluginSet),
        );
    }
}

fn handle_chunk(
    mut commands: Commands,
    q_tiles: Query<(Entity, &TileNoiseHeight), (With<TileCoord>, Without<Tile>)>,
    assets: Res<TerrainAssets>,
) {
    if q_tiles.is_empty() {
        return;
    }
    debug!("Computing Tile kind for {} tiles", q_tiles.iter().len());

    for (entity, height) in q_tiles {
        let height = **height;

        let kind = assets.get_tile(height);

        match kind {
            Some(kind) => {
                commands.entity(entity).insert(Tile(kind));
            }
            None => {
                warn!("No tile found for height: {}", height);
            }
        }
    }
}

#[derive(Resource, Debug, Clone, Default, Reflect)]
pub struct TerrainGenerationProgress {
    pub total_chunks: u32,
    pub generated_chunks: u32,
}

fn handle_chunk_progress(
    mut progress: ResMut<TerrainGenerationProgress>,
    q_chunks: Query<(&ChunkCoord, Has<ChunkReady>)>,
) {
    let total_chunks = q_chunks.iter().count() as u32;
    let generated_chunks = q_chunks.iter().filter(|(_, ready)| *ready).count() as u32;

    progress.total_chunks = total_chunks;
    progress.generated_chunks = generated_chunks;
}
