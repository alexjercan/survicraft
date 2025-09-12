use bevy::prelude::*;
use lightyear::prelude::*;
use survicraft_protocol::message::{ClientChatMessage, MessageChannel, ServerChatMessage};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ChatPluginSet;

pub(crate) struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, on_chat_message.in_set(ChatPluginSet));
    }
}

fn on_chat_message(
    mut q_receiver: Query<(&RemoteId, &mut MessageReceiver<ClientChatMessage>), Without<Client>>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
) {
    for (RemoteId(peer), mut receiver) in q_receiver.iter_mut() {
        for message in receiver.receive() {
            sender
                .send::<_, MessageChannel>(
                    &ServerChatMessage {
                        sender: *peer,
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
