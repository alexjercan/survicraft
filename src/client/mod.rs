mod chat;
mod network;

use bevy::prelude::*;

pub mod prelude {
    pub use super::network::{ClientConnection, ClientMetadata, HostConnection};
    pub use super::{ClientPlugin, ClientPluginSet};
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientPluginSet;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(network::NetworkPlugin);
        app.configure_sets(
            FixedUpdate,
            network::NetworkPluginSet.in_set(ClientPluginSet),
        );

        app.add_plugins(chat::ChatPlugin);
        app.configure_sets(FixedUpdate, chat::ChatPluginSet.in_set(ClientPluginSet));
    }
}
