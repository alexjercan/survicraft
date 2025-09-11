use bevy::prelude::*;
use lightyear::prelude::*;
use survicraft_protocol::message::{
    ClientChatMessage, MessageChannel, ServerChatMessage, ServerWelcomeMessage,
};

mod network;
mod replication;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServerPluginSet;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(network::NetworkPlugin);
        app.configure_sets(Update, network::NetworkPluginSet.in_set(ServerPluginSet));

        app.add_systems(
            Update,
            (handle_new_client, handle_chat_message).in_set(ServerPluginSet),
        );
    }
}

fn handle_new_client(
    q_connected: Query<(Entity, &RemoteId), (Added<Connected>, Without<Client>)>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
) {
    for (_, RemoteId(peer)) in q_connected.iter() {
        sender
            .send::<_, MessageChannel>(
                &ServerWelcomeMessage,
                server.clone(),
                &NetworkTarget::Single(*peer),
            )
            .unwrap_or_else(|e| {
                // TODO: Handle the error properly
                error!("Failed to send message: {:?}", e);
            });
    }
}

fn handle_chat_message(
    mut q_receiver: Query<&mut MessageReceiver<ClientChatMessage>, Without<Client>>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
) {
    for mut receiver in q_receiver.iter_mut() {
        for message in receiver.receive() {
            sender
                .send::<_, MessageChannel>(
                    &ServerChatMessage {
                        message: message.message.clone(),
                    },
                    server.clone(),
                    &NetworkTarget::All,
                )
                .unwrap_or_else(|e| {
                    // TODO: Handle the error properly
                    error!("Failed to send message: {:?}", e);
                });
        }
    }
}
