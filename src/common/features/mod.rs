mod components;
mod generation;
mod planet;
mod render;
mod resources;

use bevy::prelude::*;

use self::generation::FeaturesGenerationPlugin;

pub mod prelude {
    pub use super::components::TileFeature;
    pub use super::render::FeaturesRenderPlugin;
    pub use super::resources::*;
    pub use super::FeaturesPlugin;
}

#[derive(Default)]
pub struct FeaturesPlugin {}

impl Plugin for FeaturesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FeaturesGenerationPlugin {});
    }
}
