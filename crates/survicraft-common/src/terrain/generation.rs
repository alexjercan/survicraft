use super::{components::*, planet::*, resources::*};
use crate::{chunk_map::prelude::*, tilemap::prelude::*};
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
            .register_type::<TileNoiseHeight>();

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
