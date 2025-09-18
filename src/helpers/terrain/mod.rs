//! TODO: Add documentation
//! NOTE: If I want to make this more generic, the `Planet` resource should maybe be more of a
//! component. Instead of having it here, it should be somehow stored in the `common` crate, but
//! not sure yet how to achieve that level of genericity.

mod collider;
mod components;
mod generation;
mod geometry;
mod planet;
mod render;
mod resources;

use self::collider::{TerrainColliderPlugin, TerrainColliderPluginSet};
use self::generation::{TerrainGenerationPlugin, TerrainGenerationPluginSet};
use self::geometry::{TerrainGeometryPlugin, TerrainGeometryPluginSet};
use bevy::prelude::*;

pub mod prelude {
    pub use super::render::{TerrainRenderPlugin, TerrainRenderPluginSet};
    pub use super::resources::*;
    pub use super::{TerrainPlugin, TerrainPluginSet};
    pub use super::generation::TerrainGenerationProgress;
}

pub(crate) const TILE_SIZE: Vec2 = Vec2::splat(1.0);
pub(crate) const CHUNK_RADIUS: u32 = 16;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TerrainPluginSet;

pub struct TerrainPlugin {
    seed: u32,
    tile_size: Vec2,
    chunk_radius: u32,
    discover_radius: u32,
    max_height: f64,
}

impl Default for TerrainPlugin {
    fn default() -> Self {
        Self {
            seed: 0,
            tile_size: TILE_SIZE,
            chunk_radius: CHUNK_RADIUS,
            discover_radius: 3,
            max_height: 10.0,
        }
    }
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

    pub fn with_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
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

        app.add_plugins(TerrainColliderPlugin);
        app.configure_sets(Update, TerrainColliderPluginSet.in_set(TerrainPluginSet));
    }
}
