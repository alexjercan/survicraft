//! The Chat plugin provides a simple chat UI and handles sending and receiving chat messages.

use crate::{common::prelude::*, protocol::prelude::*};
use bevy::prelude::*;
use lightyear::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ChatPluginSet;

pub(crate) struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_chat_submit, on_chat_message).in_set(ChatPluginSet),
        );
    }
}

fn handle_chat_submit(
    mut ev_submitted: EventReader<ChatMessageSubmittedEvent>,
    mut sender: Single<&mut MessageSender<ClientChatMessage>>,
) {
    for ev in ev_submitted.read() {
        let msg = ev.message.trim();

        if !msg.is_empty() {
            sender.send::<MessageChannel>(ClientChatMessage {
                message: msg.to_string(),
            });
        }
    }
}

fn on_chat_message(
    mut receiver: Single<&mut MessageReceiver<ServerChatMessage>>,
    q_players: Query<(&PlayerName, &PlayerId), With<Replicated>>,
    mut ev_chat: EventWriter<AddChatHistoryItemEvent>,
) {
    for message in receiver.receive() {
        if let Some((name, _)) = q_players.iter().find(|(_, id)| id.0 == message.sender) {
            ev_chat.write(AddChatHistoryItemEvent {
                sender: name.0.clone(),
                message: message.message.clone(),
            });
        } else {
            warn!(
                "Received chat message from unknown player ID {:?}",
                message.sender
            );
        }
    }
}
