use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use super::main_menu::*;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum AssetState {
    #[default]
    Loading,
    Loaded,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetsPluginSet;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AssetState>();
        app.add_loading_state(
            LoadingState::new(AssetState::Loading)
                .continue_to_state(AssetState::Loaded)
                .load_collection::<MainMenuAssets>(),
        );
    }
}
