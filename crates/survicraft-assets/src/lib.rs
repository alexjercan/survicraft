use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(Debug, Clone, Event)]
pub struct AssetsLoadedEvent;

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
        app.add_event::<AssetsLoadedEvent>();

        app.init_state::<AssetState>();
        app.add_loading_state(
            LoadingState::new(AssetState::Loading)
                .continue_to_state(AssetState::Loaded)
                .load_collection::<GameAssets>(),
        );

        app.add_systems(
            OnEnter(AssetState::Loaded),
            on_assets_loaded.in_set(AssetsPluginSet),
        );
    }
}

fn on_assets_loaded(mut ev_assets: EventWriter<AssetsLoadedEvent>) {
    ev_assets.write(AssetsLoadedEvent);
    info!("Assets loaded");
}

#[derive(AssetCollection, Resource, Clone)]
pub struct GameAssets {
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
