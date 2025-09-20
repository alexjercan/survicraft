mod chat;
mod network;
mod player;

use bevy::prelude::*;

pub mod prelude {
    pub use super::network::ServerListener;
    pub use super::ServerPlugin;
}

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(network::NetworkPlugin);
        app.add_plugins(chat::ChatPlugin);
        app.add_plugins(player::PlayerPlugin);
    }
}
