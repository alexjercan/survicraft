use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ServerWelcomeMessage;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClientChatMessage {
    // NOTE: we probably need to also add the peer id or something to identify the sender
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ServerChatMessage {
    // NOTE: we probably want to add some metadata like sender id, timestamp, etc.
    pub message: String,
}

pub struct MessageChannel;

pub fn register_messages(app: &mut App) {
    // Messages for Chat
    app.add_message::<ServerWelcomeMessage>()
        .add_direction(NetworkDirection::ServerToClient);
    app.add_message::<ClientChatMessage>()
        .add_direction(NetworkDirection::ClientToServer);
    app.add_message::<ServerChatMessage>()
        .add_direction(NetworkDirection::ServerToClient);

    app.add_channel::<MessageChannel>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
        ..default()
    })
    .add_direction(NetworkDirection::Bidirectional);
}
