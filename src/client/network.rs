//! The network plugin handles client connections to the server and sync's
//! the client metadata (e.g. username) upon connection.

use crate::protocol::prelude::*;
use bevy::prelude::*;
use lightyear::{
    connection::host::HostClient,
    netcode::{Key, NetcodeClient},
    prelude::{client::NetcodeConfig, *},
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Structure representing a request to connect to a server.
/// To connect to the server, add this component to an entity.
#[derive(Debug, Clone, Component)]
pub struct ClientConnection {
    pub address: SocketAddr,
}

/// Structure representing a request to connect to the host server.
/// To connect to the host server, add this component to an entity.
#[derive(Debug, Clone, Component)]
pub struct HostConnection {
    pub server: Entity,
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
        app.add_observer(on_client_connection_added);
        app.add_observer(on_host_connection_added);
        app.add_systems(Update, on_welcome_message.in_set(NetworkPluginSet));
    }
}

fn on_client_connection_added(
    trigger: Trigger<OnAdd, ClientConnection>,
    q_connection: Query<&ClientConnection>,
    mut commands: Commands,
) -> Result {
    let entity = trigger.target();
    let connection = q_connection.get(entity)?;
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
            Link::new(None), // TODO: Check examples in lightyear
            LocalAddr(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)),
            PeerAddr(connection.address),
            ReplicationReceiver::default(),
            PredictionManager::default(),
            InterpolationManager::default(),
            NetcodeClient::new(auth, NetcodeConfig::default())?,
            UdpIo::default(),
        ))
        .id();

    commands.trigger_targets(Connect, client);

    Ok(())
}

fn on_host_connection_added(
    trigger: Trigger<OnAdd, HostConnection>,
    q_connection: Query<&HostConnection>,
    mut commands: Commands,
) -> Result {
    let entity = trigger.target();
    let connection = q_connection.get(entity)?;
    info!("Starting client, connecting to host server");

    let client = commands
        .entity(entity)
        .insert((
            Name::new("HostClient"),
            Client::default(),
            LinkOf {
                server: connection.server,
            },
            LocalAddr(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)),
            UdpIo::default(),
        ))
        .id();

    commands.trigger_targets(Connect, client);

    Ok(())
}

fn on_welcome_message(
    mut receiver: Single<
        &mut MessageReceiver<ServerWelcomeMessage>,
        Or<(With<Client>, With<HostClient>)>,
    >,
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
