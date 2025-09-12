use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bevy::prelude::*;
use lightyear::{
    netcode::{Key, NetcodeClient},
    prelude::{client::NetcodeConfig, *},
};
use survicraft_common::chat::{ChatMessageEvent, ChatPlugin, ChatPluginSet};
use survicraft_protocol::{
    PROTOCOL_ID, get_client_id,
    message::{
        ClientChatMessage, ClientMetaMessage, MessageChannel, ServerChatMessage,
        ServerWelcomeMessage,
    },
};

/// Structure representing a request to connect to a server.
/// To connect to the server, add this component to an entity.
#[derive(Debug, Clone, Component)]
pub struct ClientConnection {
    pub address: SocketAddr,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientPluginSet;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChatPlugin);
        app.configure_sets(Update, ChatPluginSet.in_set(ClientPluginSet));

        app.add_systems(
            Update,
            (
                client_connect,
                handle_welcome_message,
                on_chat_message,
                handle_chat_message,
            )
                .in_set(ClientPluginSet),
        );
    }
}

fn client_connect(
    mut commands: Commands,
    connection: Single<(Entity, &ClientConnection), Added<ClientConnection>>,
) -> Result {
    let (entity, connection) = connection.into_inner();
    info!(
        "Starting client, connecting to server at {}",
        connection.address
    );

    let auth = Authentication::Manual {
        server_addr: connection.address,
        client_id: get_client_id(),
        private_key: Key::default(),
        protocol_id: PROTOCOL_ID,
    };

    let client = commands
        .entity(entity)
        .insert((
            Name::new("Client"),
            Client::default(),
            LocalAddr(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)),
            PeerAddr(connection.address),
            Link::new(None),
            ReplicationReceiver::default(),
            NetcodeClient::new(auth, NetcodeConfig::default())?,
            UdpIo::default(),
        ))
        .id();

    commands.trigger_targets(Connect, client);

    Ok(())
}

fn handle_welcome_message(
    mut receiver: Single<&mut MessageReceiver<ServerWelcomeMessage>>,
    mut sender: Single<&mut MessageSender<ClientMetaMessage>>,
) {
    for message in receiver.receive() {
        info!("Received welcome message from server: {:?}", message);

        sender.send::<MessageChannel>(ClientMetaMessage {
            username: "Player TODO".to_string(),
        });
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
