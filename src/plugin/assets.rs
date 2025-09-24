use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use super::states::*;
use crate::prelude::*;

pub(super) struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(LauncherStates::Loading), setup_terrain_assets);
        app.add_systems(OnEnter(LauncherStates::Loading), setup_feature_assets);
        app.add_loading_state(
            LoadingState::new(LauncherStates::Loading)
                .continue_to_state(LauncherStates::MainMenu)
                // Load assets for the main menu
                .load_collection::<MainMenuAssets>(),
        );
    }
}

fn setup_terrain_assets(mut commands: Commands) {
    debug!("Setting up terrain assets...");

    // TODO: I want to load these from file, but for now, hardcode them
    // with some kind of cool syntax like:
    //
    // [terrain]
    //     id="deep_water"
    //     name="Deep Water"
    //     [generation]
    //         elevation_min=None
    //         elevation_max=0.25
    //     [/generation]
    // [/terrain]
    commands.insert_resource(TerrainAssets::new(vec![
        TileAsset {
            id: "deep_water".to_string(),
            name: "Deep Water".to_string(),
            generation: TileGeneration {
                elevation_min: None,
                elevation_max: Some(0.25),
            },
        },
        TileAsset {
            id: "shallow_water".to_string(),
            name: "Shallow Water".to_string(),
            generation: TileGeneration {
                elevation_min: Some(0.25),
                elevation_max: Some(0.5),
            },
        },
        TileAsset {
            id: "sand".to_string(),
            name: "Sand".to_string(),
            generation: TileGeneration {
                elevation_min: Some(0.5),
                elevation_max: Some(0.55),
            },
        },
        TileAsset {
            id: "grass".to_string(),
            name: "Grass".to_string(),
            generation: TileGeneration {
                elevation_min: Some(0.55),
                elevation_max: Some(0.75),
            },
        },
        TileAsset {
            id: "hills".to_string(),
            name: "Hills".to_string(),
            generation: TileGeneration {
                elevation_min: Some(0.75),
                elevation_max: Some(0.9),
            },
        },
        TileAsset {
            id: "mountain".to_string(),
            name: "Mountain".to_string(),
            generation: TileGeneration {
                elevation_min: Some(0.9),
                elevation_max: None,
            },
        },
    ]));
}

fn setup_feature_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    debug!("Setting up feature assets...");

    // TODO: I want to load these from file, but for now, hardcode them
    // with some kind of cool syntax like:
    //
    // [feature]
    //     id="tree"
    //     name="Tree"
    //     [variant]
    //         id="sand"
    //         name="Palm Tree"
    //         threshold=0.9
    //         scene="gltf/decoration/nature/trees_A_cut.gltf#Scene0"
    //     [/variant]
    //     [variant]
    //         id="grass"
    //         name="Oak Tree"
    //         threshold=0.7
    //         scene="gltf/decoration/nature/trees_A_small.gltf#Scene0"
    //     [/variant]
    //     [variant]
    //         id="hills"
    //         name="Pine Tree"
    //         threshold=0.6
    //         scene="gltf/decoration/nature/trees_A_large.gltf#Scene0"
    //     [/variant]
    //     [variant]
    //         id="mountain"
    //         name="Fir Tree"
    //         threshold=0.8
    //         scene="gltf/decoration/nature/trees_A_large.gltf#Scene0"
    //     [/variant]
    // [/feature]
    commands.insert_resource(FeatureAssets::new(vec![FeatureAsset {
        id: "tree".to_string(),
        name: "Tree".to_string(),
        variants: vec![
            FeatureVariant {
                id: "sand".to_string(),
                name: "Palm Tree".to_string(),
                threshold: 0.9,
                scene: asset_server.load("gltf/decoration/nature/tree_single_A_cut.gltf#Scene0"),
            },
            FeatureVariant {
                id: "grass".to_string(),
                name: "Oak Tree".to_string(),
                threshold: 0.7,
                scene: asset_server.load("gltf/decoration/nature/tree_single_A.gltf#Scene0"),
            },
            FeatureVariant {
                id: "hills".to_string(),
                name: "Pine Tree".to_string(),
                threshold: 0.6,
                scene: asset_server.load("gltf/decoration/nature/tree_single_B.gltf#Scene0"),
            },
            FeatureVariant {
                id: "mountain".to_string(),
                name: "Fir Tree".to_string(),
                threshold: 0.8,
                scene: asset_server.load("gltf/decoration/nature/tree_single_B.gltf#Scene0"),
            },
        ],
    }]));
}
