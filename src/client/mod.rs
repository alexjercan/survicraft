mod chat;
mod network;
mod player;

use crate::helpers::prelude::*;
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
        app.configure_sets(Update, network::NetworkPluginSet.in_set(ClientPluginSet));

        app.add_plugins(chat::ChatPlugin);
        app.configure_sets(Update, chat::ChatPluginSet.in_set(ClientPluginSet));

        app.add_plugins(TerrainRenderPlugin::default());
        app.configure_sets(Update, TerrainRenderPluginSet.in_set(ClientPluginSet));

        // app.add_plugins(player::PlayerPlugin);
        // app.configure_sets(FixedUpdate, player::PlayerPluginSet.in_set(ClientPluginSet));
        // app.configure_sets(Update, player::PlayerPluginSet.in_set(ClientPluginSet));
    }
}
