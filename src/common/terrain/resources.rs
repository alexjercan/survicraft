//! TODO: Add description

use bevy::prelude::*;

#[derive(Resource, Clone, Default, Debug)]
pub struct TerrainAssets {
    pub tiles: Vec<TileAsset>,
}

impl TerrainAssets {
    pub fn new(tiles: Vec<TileAsset>) -> Self {
        Self { tiles }
    }

    pub fn get_tile(&self, elevation: f64) -> Option<TileID> {
        self.tiles
            .iter()
            .find(|tile| {
                tile.generation
                    .elevation_min
                    .is_none_or(|min| elevation >= min)
                    && tile
                        .generation
                        .elevation_max
                        .is_none_or(|max| elevation <= max)
            })
            .map(|tile| tile.id.clone())
    }

    pub fn get_tile_index(&self, id: &TileID) -> Option<usize> {
        self.tiles.iter().position(|tile| &tile.id == id)
    }
}

pub type TileID = String;

#[derive(Debug, Clone)]
pub struct TileAsset {
    pub id: TileID,
    pub name: String,
    pub generation: TileGeneration,
}

#[derive(Debug, Clone)]
pub struct TileGeneration {
    pub elevation_min: Option<f64>,
    pub elevation_max: Option<f64>,
}
