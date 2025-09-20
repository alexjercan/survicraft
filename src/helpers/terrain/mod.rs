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

use self::collider::TerrainColliderPlugin;
use self::generation::TerrainGenerationPlugin;
use self::geometry::TerrainGeometryPlugin;
use bevy::prelude::*;

pub mod prelude {
    pub use super::generation::{TerrainGenerationProgress, TerrainGenerationSeed};
    pub use super::render::TerrainRenderPlugin;
    pub use super::resources::*;
    pub use super::TerrainPlugin;
}

pub(crate) const TILE_SIZE: Vec2 = Vec2::splat(1.0);
pub(crate) const CHUNK_RADIUS: u32 = 16;

pub struct TerrainPlugin {
    tile_size: Vec2,
    chunk_radius: u32,
    discover_radius: u32,
    max_height: f64,
}

impl Default for TerrainPlugin {
    fn default() -> Self {
        Self {
            tile_size: TILE_SIZE,
            chunk_radius: CHUNK_RADIUS,
            discover_radius: 3,
            max_height: 10.0,
        }
    }
}

impl TerrainPlugin {
    pub fn new(tile_size: Vec2, chunk_radius: u32, discover_radius: u32, max_height: f64) -> Self {
        Self {
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
            self.tile_size,
            self.chunk_radius,
            self.discover_radius,
        ));
        app.add_plugins(TerrainGeometryPlugin::new(self.tile_size, self.max_height));
        app.add_plugins(TerrainColliderPlugin);
    }
}
