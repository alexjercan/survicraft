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

        // Physics setup. We disable interpolation and sleeping to ensure consistent physics
        app.add_plugins(
            PhysicsPlugins::default()
                .build()
                .disable::<PhysicsInterpolationPlugin>()
                // disable Sleeping plugin as it can mess up physics rollbacks
                .disable::<SleepingPlugin>(),
        );

        // Terrain setup. We set up terrain assets and the terrain plugin itself.
        // This will run only in the Playing state.
        app.add_plugins(TerrainPlugin::default().with_seed(0));
        app.configure_sets(Update, TerrainPluginSet);

        // Player setup. We set up player-related systems and the player plugin.
        app.add_plugins(PlayerPlugin);
        app.configure_sets(FixedUpdate, PlayerPluginSet);
        app.configure_sets(Update, PlayerPluginSet);

        // --- Server plugins below here ---

        // The server plugin will run only if we are the server (i.e. hosting)
        // and in the Playing state
        app.add_systems(Startup, setup_server);
        app.add_plugins(ServerPlugin);
        app.configure_sets(FixedUpdate, ServerPluginSet);

        // NOTE: For debugging purposes
        app.add_systems(Startup, create_a_single_test_chunk);
    }
}

fn setup_server(mut commands: Commands) {
    commands.spawn((Name::new("ServerListener"), ServerListener));
}

fn create_a_single_test_chunk(mut ev_discover: EventWriter<TileDiscoverEvent>) {
    ev_discover.write(TileDiscoverEvent::new(Vec2::ZERO));
}
