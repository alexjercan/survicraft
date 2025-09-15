//! A Bevy plugin that serves as the main entry point for the game launcher.

use super::{assets::*, main_menu::*};
use crate::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_simple_text_input::TextInputPlugin;
use lightyear::{
    connection::identity::is_server,
    prelude::{client::ClientPlugins, server::ServerPlugins},
};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    time::Duration,
};

// #[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
// enum LauncherStates {
//     #[default]
//     Loading,
//     MainMenu,
//     Playing,
// }

pub struct DedicatedServerPlugin;

impl Plugin for DedicatedServerPlugin {
    fn build(&self, app: &mut App) {
        // // Initialize the state machine
        // app.init_state::<LauncherStates>();
        // app.enable_state_scoped_entities::<LauncherStates>();

        app.add_plugins(ServerPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        });

        // Protocol plugin for handling message serialization and deserialization.
        app.add_plugins(ProtocolPlugin);

        // // Physics setup. We disable interpolation and sleeping to ensure consistent physics
        // app.add_plugins(
        //     PhysicsPlugins::default()
        //         .build()
        //         .disable::<PhysicsInterpolationPlugin>()
        //         // disable Sleeping plugin as it can mess up physics rollbacks
        //         .disable::<SleepingPlugin>(),
        // );

        // // Terrain setup. We set up terrain assets and the terrain plugin itself.
        // // This will run only in the Playing state.
        // app.add_systems(OnEnter(LauncherStates::Playing), setup_terrain);
        // app.add_plugins(TerrainPlugin::default().with_seed(0));
        // app.configure_sets(
        //     Update,
        //     TerrainPluginSet.run_if(in_state(LauncherStates::Playing)),
        // );

        // The server plugin will run only if we are the server (i.e. hosting)
        // and in the Playing state
        app.add_systems(Startup, setup_server);
        app.add_plugins(ServerPlugin);
        app.configure_sets(FixedUpdate, ServerPluginSet);
        app.configure_sets(Update, ServerPluginSet);
    }
}

fn setup_server(mut commands: Commands) {
    commands.spawn((Name::new("ServerListener"), ServerListener));
}
