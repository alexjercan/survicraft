//! The Chat plugin provides a simple chat UI and handles sending and receiving chat messages.

use crate::{helpers::prelude::*, protocol::prelude::*};
use bevy::prelude::*;
use lightyear::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ChatPluginSet;

pub(crate) struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
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
        debug!("Player submitted chat message: {}", msg);

        if !msg.is_empty() {
            sender.send::<MessageChannel>(ClientChatMessage {
                message: msg.to_string(),
            });
        }
    }
}

fn on_chat_message(
    mut ev_chat: EventReader<ServerChatMessageEvent>,
    q_players: Query<(&PlayerName, &PlayerId), With<Replicated>>,
    mut ev_history: EventWriter<AddChatHistoryItemEvent>,
) {
    for ev in ev_chat.read() {
        if let Some((name, _)) = q_players.iter().find(|(_, id)| id.0 == ev.sender) {
            debug!("Received chat message from {}: {}", name.0, ev.message);

            ev_history.write(AddChatHistoryItemEvent {
                sender: name.0.clone(),
                message: ev.message.clone(),
            });
        } else {
            warn!(
                "Received chat message from unknown player ID {:?}",
                ev.sender
            );
        }
    }
}
