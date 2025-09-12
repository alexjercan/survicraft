mod chat;
mod network;

use bevy::prelude::*;
use lightyear::prelude::*;
use survicraft_protocol::{
    component::{PlayerId, PlayerName},
    message::ClientMetaMessage,
};

pub use network::ServerListener;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServerPluginSet;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(chat::ChatPlugin);
        app.configure_sets(Update, chat::ChatPluginSet.in_set(ServerPluginSet));

        app.add_plugins(network::NetworkPlugin);
        app.configure_sets(Update, network::NetworkPluginSet.in_set(ServerPluginSet));

        app.add_systems(Update, (handle_spawn_player,).in_set(ServerPluginSet));
    }
}

fn handle_spawn_player(
    mut commands: Commands,
    mut q_receiver: Query<
        (Entity, &RemoteId, &mut MessageReceiver<ClientMetaMessage>),
        Without<Client>,
    >,
) {
    for (entity, RemoteId(peer), mut receiver) in q_receiver.iter_mut() {
        for message in receiver.receive() {
            info!("Client {:?} set their name to {}", peer, message.username);

            commands.spawn((
                Name::new(format!("Player {}", message.username)),
                PlayerName(message.username.clone()),
                PlayerId(*peer),
                Replicate::to_clients(NetworkTarget::All),
                PredictionTarget::to_clients(NetworkTarget::Single(*peer)),
                InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(*peer)),
                ControlledBy {
                    owner: entity,
                    lifetime: Lifetime::default(),
                },
            ));
        }
    }
}
