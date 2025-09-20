//! A Bevy plugin that serves as the main entry point for the game launcher.

use crate::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::server::ServerPlugins;
use std::time::Duration;

pub struct DedicatedServerPlugin;

impl Plugin for DedicatedServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ServerPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        });

        // Protocol plugin for handling message serialization and deserialization.
        app.add_plugins(ProtocolPlugin);

        // --- Playing related stuff below here ---

        app.add_systems(Startup, (setup_terrain_assets, setup_feature_assets, setup_terrain_generation).chain());
        app.add_systems(Startup, setup_server);

        // Terrain setup. We set up terrain assets and the terrain plugin itself.
        // This will run only in the Playing state.
        app.add_plugins(TerrainGenerationPlugin { render: false });
        app.add_plugins(FeaturesGenerationPlugin { render: false });

        // Physics setup. We disable interpolation and sleeping to ensure consistent physics
        app.add_plugins(
            PhysicsPlugins::default()
                .build()
                .disable::<PhysicsInterpolationPlugin>()
                // disable Sleeping plugin as it can mess up physics rollbacks
                .disable::<SleepingPlugin>(),
        );

        // Player setup. We set up player-related systems and the player plugin.
        app.add_plugins(PlayerPlugin);

        // --- Server plugins below here ---

        // The server plugin will run only if we are the server (i.e. hosting)
        // and in the Playing state
        app.add_plugins(ServerPlugin);
    }
}

fn setup_server(mut commands: Commands) {
    commands.spawn((Name::new("ServerListener"), ServerListener));
}

fn setup_terrain_generation(mut commands: Commands) {
    commands.spawn((
        Name::new("InitializeTerrain"),
        InitializeTerrain,
    ));
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
                scene: asset_server.load("gltf/decoration/nature/trees_A_cut.gltf#Scene0"),
            },
            FeatureVariant {
                id: "grass".to_string(),
                name: "Oak Tree".to_string(),
                threshold: 0.3,
                scene: asset_server.load("gltf/decoration/nature/trees_A_small.gltf#Scene0"),
            },
            FeatureVariant {
                id: "hills".to_string(),
                name: "Pine Tree".to_string(),
                threshold: 0.4,
                scene: asset_server.load("gltf/decoration/nature/trees_A_large.gltf#Scene0"),
            },
            FeatureVariant {
                id: "mountain".to_string(),
                name: "Fir Tree".to_string(),
                threshold: 0.6,
                scene: asset_server.load("gltf/decoration/nature/trees_A_large.gltf#Scene0"),
            },
        ],
    }]));
}
