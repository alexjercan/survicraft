//! TODO: Add description

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource, Clone)]
pub struct MainMenuAssets {
    #[asset(path = "textures/GameIcons/exitRight.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub exit_icon: Handle<Image>,
    #[asset(path = "textures/GameIcons/right.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub right_icon: Handle<Image>,
    #[asset(path = "textures/GameIcons/wrench.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub wrench_icon: Handle<Image>,
}
