use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use super::states::*;
use crate::prelude::*;

pub(super) struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(LauncherStates::Loading), setup_terrain_assets);
        app.add_systems(OnEnter(LauncherStates::Loading), setup_feature_assets);
        app.add_systems(OnEnter(LauncherStates::Loading), setup_item_assets);
        app.add_systems(OnEnter(LauncherStates::Loading), setup_recipe_assets);
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

fn setup_item_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    debug!("Setting up item assets...");

    // TODO: I want to load these from file, but for now, hardcode them
    // with some kind of cool syntax like:
    //
    // [item]
    //     id = "wood"
    //     name = "Wood"
    //     description = "A piece of wood."
    //     weight = 10.0
    //     scene="gltf/decoration/props/resource_lumber.gltf#Scene0"
    // [/item]
    // [item]
    //     id = "wood_box"
    //     name = "Wood Box"
    //     description = "A storage box made of wood."
    //     weight = 50.0
    //     scene="gltf/decoration/props/crate_A_big.gltf#Scene0"
    //     [modifier]
    //         type = storage
    //         capacity = 100
    //     [/modifier]
    // [/item]
    commands.insert_resource(ItemAssets::new(vec![
        ItemAsset {
            id: "wood".to_string(),
            name: "Wood".to_string(),
            description: "A piece of wood.".to_string(),
            weight: 10.0,
            scene: asset_server.load("gltf/decoration/props/resource_lumber.gltf#Scene0"),
            offset: Some(Vec3::new(0.0, -0.1, 0.0)),
            scale: Some(Vec3::splat(1.0)),
            collider: Some(Collider::cuboid(0.75, 0.2, 0.4)),
            modifiers: vec![],
        },
        ItemAsset {
            id: "wood_box".to_string(),
            name: "Wood Box".to_string(),
            description: "A storage box made of wood.".to_string(),
            weight: 50.0,
            scene: asset_server.load("gltf/decoration/props/crate_A_big.gltf#Scene0"),
            offset: Some(Vec3::new(0.0, -0.2, 0.0)),
            scale: Some(Vec3::splat(2.0)),
            collider: Some(Collider::cuboid(0.5, 0.5, 0.5)),
            modifiers: vec![ItemModifier::Storage { capacity: 100 }],
        },
    ]));
}

fn setup_recipe_assets(mut commands: Commands) {
    debug!("Setting up recipe assets...");

    // TODO: I want to load these from file, but for now, hardcode them
    // with some kind of cool syntax like:
    //
    // [recipe]
    //    [input]
    //        item = "wood"
    //        count = 5
    //    [/input]
    //    [output]
    //        item = "wood_box"
    //        count = 1
    //    [/output]
    // [/recipe]
    commands.insert_resource(RecipeAssets::new(vec![RecipeAsset {
        input: vec![RecipeItem {
            item_id: "wood".to_string(),
            count: 5,
        }],
        output: vec![RecipeItem {
            item_id: "wood_box".to_string(),
            count: 1,
        }],
    }]));
}
