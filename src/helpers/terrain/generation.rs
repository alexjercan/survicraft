//! TODO: Documentation

use std::time::SystemTime;

use super::{components::*, planet::*, resources::*};
use crate::helpers::{chunk_map::prelude::*, tilemap::prelude::*};
use bevy::prelude::*;

#[derive(Resource, Debug, Clone, PartialEq, Deref, DerefMut, Reflect)]
pub struct TerrainGenerationSeed(pub u32);

impl Default for TerrainGenerationSeed {
    fn default() -> Self {
        Self(
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as u32,
        )
    }
}

pub struct TerrainGenerationPlugin {
    tile_size: Vec2,
    chunk_radius: u32,
}

impl TerrainGenerationPlugin {
    pub fn new(tile_size: Vec2, chunk_radius: u32) -> Self {
        Self {
            tile_size,
            chunk_radius,
        }
    }
}

impl Plugin for TerrainGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Tile>()
            .register_type::<TileNoiseHeight>()
            .register_type::<TerrainGenerationProgress>()
            .register_type::<TerrainGenerationSeed>();

        app.add_plugins(TileMapPlugin::new(self.tile_size, self.chunk_radius))
            // TODO: Somehow I want to be able to add this PlanetHeight from outside
            // I need this to be more generic for copy pastability
            .add_plugins(ChunkMapPlugin::<TileCoord, TileNoiseHeight, _>::new(
                PlanetHeight::default(),
            ))
            .add_systems(Update, handle_chunk);

        app.insert_resource(TerrainGenerationProgress::default());
        app.add_systems(Update, handle_chunk_progress);

        app.insert_resource(TerrainGenerationSeed::default());
        app.add_systems(
            Update,
            update_terrain_seed.run_if(resource_changed::<TerrainGenerationSeed>),
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
    trace!("Computing Tile kind for {} tiles", q_tiles.iter().len());

    for (entity, height) in q_tiles {
        let height = **height;

        let kind = assets.get_tile(height);

        match kind {
            Some(kind) => {
                commands.entity(entity).insert(Tile(kind));
            }
            None => {
                error!("No tile found for height: {}", height);
            }
        }
    }
}

fn update_terrain_seed(mut func: ResMut<PlanetHeight>, seed: Res<TerrainGenerationSeed>) {
    *func = func.with_seed(**seed);
    debug!("Updated terrain seed to {}", seed.0);
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
