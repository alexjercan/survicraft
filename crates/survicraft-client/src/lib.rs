use bevy::prelude::*;

mod network;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientPluginSet;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(network::NetworkPlugin);
        app.configure_sets(Update, network::NetworkPluginSet.in_set(ClientPluginSet));
    }
}
