use bevy::prelude::*;

mod network;
mod replication;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(network::NetworkPlugin);
    }
}
