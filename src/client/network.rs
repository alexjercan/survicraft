//! The network plugin handles client connections to the server and sync's
//! the client metadata (e.g. username) upon connection.

use crate::{helpers::prelude::*, protocol::prelude::*};
use bevy::prelude::*;
use lightyear::{
    netcode::{Key, NetcodeClient},
    prelude::{client::NetcodeConfig, *},
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::SystemTime;

/// Structure representing a request to connect to a server.
/// To connect to the server, add this component to an entity.
#[derive(Debug, Clone, Component, Reflect)]
pub struct ClientConnection {
    pub address: SocketAddr,
}

/// Structure representing a request to connect to the host server.
/// To connect to the host server, add this component to an entity.
#[derive(Debug, Clone, Component, Reflect)]
pub struct HostConnection {
    pub server: Entity,
}

#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Deref, DerefMut)]
pub struct PlayerNameSetting(pub String);

#[derive(Resource, Debug, Clone, PartialEq, Eq, Deref, DerefMut, Reflect)]
pub struct ClientNetworkStateReady(pub bool);

impl Default for PlayerNameSetting {
    fn default() -> Self {
        let time = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self(format!("Player{}", time % 1000))
    }
}

pub(crate) struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ClientConnection>()
            .register_type::<HostConnection>();

        app.insert_resource(PlayerNameSetting::default());
        app.insert_resource(ClientNetworkStateReady(false));

        app.add_observer(on_client_connection_added);
        app.add_observer(on_host_connection_added);
        app.add_systems(FixedUpdate, on_welcome_message);
    }
}

fn on_client_connection_added(
    trigger: Trigger<OnAdd, ClientConnection>,
    q_connection: Query<&ClientConnection, Added<ClientConnection>>,
    mut commands: Commands,
    mut client_ready: ResMut<ClientNetworkStateReady>,
) -> Result {
    let entity = trigger.target();
    let connection = q_connection.get(entity)?;
    info!(
        "Starting client, connecting to server at {}",
        connection.address
    );
    **client_ready = false;

    let auth = Authentication::Manual {
        server_addr: connection.address,
        client_id: get_client_id(),
        private_key: Key::default(),
        protocol_id: PROTOCOL_ID,
    };

    let conditioner = LinkConditionerConfig::average_condition();
    let client = commands
        .entity(entity)
        .insert((
            Name::new("Client"),
            Client::default(),
            Link::new(Some(RecvLinkConditioner::new(conditioner.clone()))),
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
    mut client_ready: ResMut<ClientNetworkStateReady>,
) -> Result {
    let entity = trigger.target();
    let connection = q_connection.get(entity)?;
    info!("Starting client, connecting to host server");
    **client_ready = false;

    let client = commands
        .entity(entity)
        .insert((
            Name::new("HostClient"),
            Client::default(),
            LinkOf {
                server: connection.server,
            },
        ))
        .id();

    commands.trigger_targets(Connect, client);

    Ok(())
}

fn on_welcome_message(
    mut ev_welcome: EventReader<ServerWelcomeEvent>,
    sender: Single<(&RemoteId, &mut MessageSender<ClientMetaMessage>)>,
    player_name: Res<PlayerNameSetting>,
    mut client_ready: ResMut<ClientNetworkStateReady>,
    mut world_seed: ResMut<TerrainGenerationSeed>,
) {
    let (RemoteId(local), mut sender) = sender.into_inner();
    for &ServerWelcomeEvent { peer, seed } in ev_welcome.read() {
        if *local != peer {
            continue;
        }
        debug!("Received welcome message from server: {:?}", peer);
        **client_ready = true;
        **world_seed = seed;

        let metadata = ClientMetaMessage {
            username: player_name.to_string(),
        };
        debug!("Sending client metadata: {:?}", metadata);
        sender.send::<MessageChannel>(metadata);
    }
}
