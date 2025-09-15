mod chat;
mod network;
mod player;

use bevy::prelude::*;
use crate::helpers::prelude::*;

pub mod prelude {
    pub use super::{ServerPlugin, ServerPluginSet};
    pub use super::network::ServerListener;
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServerPluginSet;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(chat::ChatPlugin);
        app.configure_sets(Update, chat::ChatPluginSet.in_set(ServerPluginSet));

        app.add_plugins(network::NetworkPlugin);
        app.configure_sets(Update, network::NetworkPluginSet.in_set(ServerPluginSet));

        app.add_plugins(player::PlayerPlugin);
        app.configure_sets(FixedUpdate, player::PlayerPluginSet.in_set(ServerPluginSet));
        app.configure_sets(Update, player::PlayerPluginSet.in_set(ServerPluginSet));

        app.add_systems(
            Update,
            create_a_single_test_chunk.in_set(ServerPluginSet),
        );
    }
}

fn create_a_single_test_chunk(mut ev_discover: EventWriter<TileDiscoverEvent>) {
    ev_discover.send(TileDiscoverEvent::new(Vec2::ZERO));
}
