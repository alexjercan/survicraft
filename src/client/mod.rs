mod chat;
mod network;
mod player;

// For debugging purposes
mod controller;

use crate::{helpers::prelude::*, prelude::ClientConnection};
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

        // // For debugging purposes
        // app.add_plugins(controller::WASDCameraControllerPlugin);
        // app.configure_sets(
        //     Update,
        //     controller::WASDCameraControllerPluginSet.in_set(ClientPluginSet),
        // );
    }
}

fn create_a_single_test_chunk(mut ev_discover: EventWriter<TileDiscoverEvent>) {
    ev_discover.send(TileDiscoverEvent::new(Vec2::ZERO));
}
