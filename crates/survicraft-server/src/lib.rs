use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::server::ServerPlugins;
use survicraft_protocol::FIXED_TIMESTEP_HZ;

mod network;
mod replication;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ServerPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        });

        app.add_plugins(network::NetworkPlugin);
    }
}
