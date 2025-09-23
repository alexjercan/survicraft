mod components;
mod generation;
mod planet;
mod render;
mod resources;

use self::generation::FeaturesGenerationPlugin;
use bevy::prelude::*;

pub mod prelude {
    pub use super::components::TileFeature;
    pub use super::resources::*;
    pub use super::FeaturesPlugin;
    pub use super::render::FeaturesRenderPlugin;
}

pub struct FeaturesPlugin {}

impl Default for FeaturesPlugin {
    fn default() -> Self {
        Self {}
    }
}

impl Plugin for FeaturesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FeaturesGenerationPlugin {});
    }
}
