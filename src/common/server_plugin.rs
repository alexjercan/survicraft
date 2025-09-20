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

        app.add_systems(Startup, (setup_terrain_assets, setup_terrain_generation).chain());
        app.add_systems(Startup, setup_server);

        // Terrain setup. We set up terrain assets and the terrain plugin itself.
        // This will run only in the Playing state.
        app.add_plugins(TerrainPlugin::default());

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

fn setup_terrain_generation(mut ev_discover: EventWriter<TileDiscoverEvent>) {
    ev_discover.write(TileDiscoverEvent::new(Vec2::ZERO, 5));
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
