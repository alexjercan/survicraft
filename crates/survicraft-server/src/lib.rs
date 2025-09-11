use bevy::prelude::*;

mod network;
mod replication;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServerPluginSet;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(network::NetworkPlugin);
        app.configure_sets(Update, network::NetworkPluginSet.in_set(ServerPluginSet));
    }
}
