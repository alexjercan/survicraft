use bevy::prelude::*;
use lightyear::prelude::*;
use survicraft_common::chat::{ChatMessageEvent, ChatPlugin, ChatPluginSet};
use survicraft_protocol::message::{
    ClientChatMessage, MessageChannel, ServerChatMessage, ServerWelcomeMessage,
};

mod network;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientPluginSet;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(network::NetworkPlugin);
        app.configure_sets(Update, network::NetworkPluginSet.in_set(ClientPluginSet));

        app.add_plugins(ChatPlugin);
        app.configure_sets(Update, ChatPluginSet.in_set(ClientPluginSet));

        app.add_systems(
            Update,
            (handle_welcome_message, on_chat_message, handle_chat_message).in_set(ClientPluginSet),
        );
    }
}

fn handle_welcome_message(mut receiver: Single<&mut MessageReceiver<ServerWelcomeMessage>>) {
    for message in receiver.receive() {
        info!("Received welcome message from server: {:?}", message);
    }
}

fn on_chat_message(
    mut ev_chat: EventReader<ChatMessageEvent>,
    mut sender: Single<&mut MessageSender<ClientChatMessage>>,
) {
    for ChatMessageEvent(message) in ev_chat.read() {
        sender.send::<MessageChannel>(ClientChatMessage {
            message: message.clone(),
        });
    }
}

fn handle_chat_message(mut receiver: Single<&mut MessageReceiver<ServerChatMessage>>) {
    for message in receiver.receive() {
        info!("Received chat message from server: {:?}", message.message);
    }
}
