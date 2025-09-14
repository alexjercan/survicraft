//! The Terrain plugin provides systems and resources for managing and rendering the game terrain.

use bevy::prelude::*;
use survicraft_common::terrain::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TerrainPluginSet;

pub(crate) struct TerrainPlugin {
    seed: u32,
    tile_size: Vec2,
    chunk_radius: u32,
    discover_radius: u32,
    max_height: f64,
}

impl TerrainPlugin {
    pub fn new(
        seed: u32,
        tile_size: Vec2,
        chunk_radius: u32,
        discover_radius: u32,
        max_height: f64,
    ) -> Self {
        Self {
            seed,
            tile_size,
            chunk_radius,
            discover_radius,
            max_height,
        }
    }
}

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TerrainGenerationPlugin::new(
            self.seed,
            self.tile_size,
            self.chunk_radius,
            self.discover_radius,
        ));
        app.configure_sets(Update, TerrainGenerationPluginSet.in_set(TerrainPluginSet));
        app.add_plugins(TerrainGeometryPlugin::new(self.tile_size, self.max_height));
        app.configure_sets(Update, TerrainGeometryPluginSet.in_set(TerrainPluginSet));
        app.add_plugins(TerrainRenderPlugin::new(self.tile_size, self.chunk_radius));
        app.configure_sets(Update, TerrainRenderPluginSet.in_set(TerrainPluginSet));

        // testing
        app.add_plugins(TerrainColliderPlugin);
        app.configure_sets(Update, TerrainColliderPluginSet.in_set(TerrainPluginSet));
    }
}
