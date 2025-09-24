use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    time::Duration,
};

use bevy::prelude::*;
use lightyear::{
    netcode::{Key, NetcodeClient, NetcodeServer},
    prelude::{
        client::ClientPlugins,
        server::{ClientOf, ServerPlugins, ServerUdpIo, Start},
        *,
    },
};

use super::{protocol::*, resources::*, states::*};
use crate::prelude::*;

pub(super) struct NetworkPlugin {
    pub dedicated: bool,
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        // Multiplayer setup for lightyear. Here we set up both client and server plugins,
        // but we will control which ones are active using states and conditions. In case
        // the player is hosting, both client and server will be active. In case the player
        // is joining, only the client will be active.
        if !self.dedicated {
            app.add_plugins(ClientPlugins {
                tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
            });
        }
        app.add_plugins(ServerPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        });

        app.add_plugins(ProtocolPlugin);

        if !self.dedicated {
            app.add_plugins(ClientPlugin);
        }
        app.add_plugins(ServerPlugin);

        // Create connections when entering the Connecting state.
        app.add_systems(OnEnter(LauncherStates::Connecting), setup_connections);
    }
}

fn setup_connections(mut commands: Commands, mode: Res<LauncherMode>) {
    info!("Setting up connections in mode: {:?}", *mode);

    match &*mode {
        LauncherMode::Host => {
            let server = commands
                .spawn((
                    Name::new("ServerListener"),
                    ServerListener,
                    StateScoped(LauncherStates::Playing),
                ))
                .id();

            commands.spawn((
                Name::new("ClientConnection"),
                HostConnection { server },
                StateScoped(LauncherStates::Playing),
            ));
        }
        LauncherMode::Client(address) => {
            let addr = IpAddr::from_str(address).unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));

            commands.spawn((
                Name::new("ClientConnection"),
                ClientConnection {
                    address: SocketAddr::new(addr, SERVER_PORT),
                },
                StateScoped(LauncherStates::Playing),
            ));
        }
        LauncherMode::Dedicated => {
            commands.spawn((
                Name::new("ServerListener"),
                ServerListener,
                StateScoped(LauncherStates::Playing),
            ));
        }
    }
}

/// Structure representing a request to connect to a server.
/// To connect to the server, add this component to an entity.
#[derive(Debug, Clone, Component, Reflect)]
struct ClientConnection {
    pub address: SocketAddr,
}

/// Structure representing a request to connect to the host server.
/// To connect to the host server, add this component to an entity.
#[derive(Debug, Clone, Component, Reflect)]
struct HostConnection {
    pub server: Entity,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq, Deref, DerefMut, Reflect)]
pub struct ClientNetworkStateReady(pub bool);

struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ClientConnection>()
            .register_type::<HostConnection>();

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
            NetcodeClient::new(auth, client::NetcodeConfig::default())?,
            UdpIo::default(),
        ))
        .insert(ReplicationSender::new(
            SERVER_REPLICATION_INTERVAL,
            SendUpdatesMode::SinceLastAck,
            false,
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

/// Server component to mark the server entity.
/// Add this component to an entity to make it a server.
#[derive(Debug, Clone, Component)]
struct ServerListener;

struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_server_listener_added);
        app.add_observer(on_new_client);
        app.add_observer(on_new_connection);

        app.add_systems(FixedUpdate, on_client_metadata_message);
    }
}

fn on_server_listener_added(trigger: Trigger<OnAdd, ServerListener>, mut commands: Commands) {
    info!("Starting server on {}", SERVER_ADDR);

    let server = commands
        .entity(trigger.target())
        .insert((
            Name::new("Server"),
            NetcodeServer::new(server::NetcodeConfig::default().with_protocol_id(PROTOCOL_ID)),
            LocalAddr(SERVER_ADDR),
            ServerUdpIo::default(),
        ))
        .id();

    commands.trigger_targets(Start, server);
}

fn on_new_client(
    trigger: Trigger<OnAdd, LinkOf>,
    mut commands: Commands,
    _server: Single<&Server>,
) {
    info!("New client connected: {:?}", trigger.target());

    commands
        .entity(trigger.target())
        .insert(Name::new("Client"))
        .insert(ReplicationReceiver::default())
        .insert(ReplicationSender::new(
            SERVER_REPLICATION_INTERVAL,
            SendUpdatesMode::SinceLastAck,
            false,
        ));
}

fn on_new_connection(
    trigger: Trigger<OnAdd, Connected>,
    q_connected: Query<&RemoteId, With<ClientOf>>,
    mut ev_welcome: EventWriter<ServerWelcomeEvent>,
    world_seed: Res<TerrainGenerationSeed>,
    _: Single<&Server>,
) -> Result {
    info!("New connection established: {:?}", trigger.target());

    let entity = trigger.target();
    let RemoteId(peer) = q_connected.get(entity)?;
    let welcome = ServerWelcomeEvent {
        peer: *peer,
        seed: **world_seed,
    };
    debug!("Sending welcome message to {:?}: {:?}", peer, welcome);

    ev_welcome.write(welcome);

    Ok(())
}

fn on_client_metadata_message(
    mut commands: Commands,
    mut q_receiver: Query<(&RemoteId, &mut MessageReceiver<ClientMetaMessage>)>,
) {
    for (RemoteId(peer), mut receiver) in q_receiver.iter_mut() {
        for message in receiver.receive() {
            debug!("Spawn player metadata for peer {:?}: {:?}", peer, message);

            commands.spawn((
                Name::new("PlayerMetadata"),
                PlayerId(*peer),
                PlayerMetadata {
                    username: message.username.clone(),
                },
                Replicate::to_clients(NetworkTarget::All),
            ));
        }
    }
}
