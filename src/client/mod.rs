mod chat;
mod network;
mod player;

use bevy::prelude::*;

pub mod prelude {
    pub use super::network::{
        ClientConnection, ClientNetworkStateReady, HostConnection, PlayerNameSetting,
    };
    pub use super::player::ClientSpawnPlayerEvent;
    pub use super::ClientPlugin;
}

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(network::NetworkPlugin);
        app.add_plugins(chat::ChatPlugin);
        app.add_plugins(player::PlayerPlugin);
    }
}
