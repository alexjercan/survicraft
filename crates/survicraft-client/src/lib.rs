use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::client::ClientPlugins;
use survicraft_protocol::FIXED_TIMESTEP_HZ;

mod network;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ClientPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        });

        app.add_plugins(network::NetworkPlugin);
    }
}
