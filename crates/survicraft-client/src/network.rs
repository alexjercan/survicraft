//! The network plugin handles client connections to the server and sync's
//! the client metadata (e.g. username) upon connection.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bevy::prelude::*;
use lightyear::{netcode::{Key, NetcodeClient}, prelude::{client::NetcodeConfig, *}};
use survicraft_protocol::{get_client_id, message::{ClientMetaMessage, MessageChannel, ServerWelcomeMessage}, PROTOCOL_ID};

/// Structure representing a request to connect to a server.
/// To connect to the server, add this component to an entity.
#[derive(Debug, Clone, Component)]
pub struct ClientConnection {
    pub address: SocketAddr,
}

/// Component to store client metadata such as username.
/// This is used to add metadata to the client upon connection.
#[derive(Debug, Clone, Component)]
pub struct ClientMetadata {
    pub username: String,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct NetworkPluginSet;

pub(crate) struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (on_client_connection_added, on_welcome_message).in_set(NetworkPluginSet),
        );
    }
}

fn on_client_connection_added(
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

fn on_welcome_message(
    mut receiver: Single<&mut MessageReceiver<ServerWelcomeMessage>>,
    mut sender: Single<&mut MessageSender<ClientMetaMessage>>,
    metadata: Single<&ClientMetadata>,
) {
    for message in receiver.receive() {
        info!("Received welcome message from server: {:?}", message);

        sender.send::<MessageChannel>(ClientMetaMessage {
            username: metadata.username.clone(),
        });
    }
}
