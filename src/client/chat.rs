//! The Chat plugin provides a simple chat UI and handles sending and receiving chat messages.

use crate::{helpers::prelude::*, protocol::prelude::*};
use bevy::prelude::*;
use lightyear::prelude::*;

pub(crate) struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_chat_submit, on_chat_message));
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
    q_players: Query<(&PlayerMetadata, &PlayerId), With<Replicated>>,
    mut ev_history: EventWriter<AddChatHistoryItemEvent>,
) {
    for ev in ev_chat.read() {
        if let Some((PlayerMetadata { username, .. }, _)) =
            q_players.iter().find(|(_, id)| id.0 == ev.sender)
        {
            debug!("Received chat message from {}: {}", username, ev.message);

            ev_history.write(AddChatHistoryItemEvent {
                sender: username.clone(),
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
