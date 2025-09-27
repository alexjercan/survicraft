mod helpers;

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use bevy::prelude::*;
use clap::Parser;
use lightyear::{
    connection::host::HostClient, netcode::{Key, NetcodeClient, NetcodeServer}, prelude::{
        client::ClientPlugins,
        server::{ClientOf, ServerPlugins, ServerUdpIo, Start},
        *,
    }
};
use serde::{Deserialize, Serialize};
use survicraft::prelude::*;

#[derive(Parser)]
#[command(name = "survicraft-network")]
#[command(version = "0.1")]
#[command(about = "Example for the survicraft network implementation", long_about = None)]
struct Cli {
    #[arg(short = 'H', long = "host")]
    host: bool,
}

fn main() {
    let cli = Cli::parse();

    let mut app = new_gui_app();
    app.add_plugins(NetworkPlugin { host: cli.host });

    app.run();
}

const PROTOCOL_ID: u64 = 0;
const FIXED_TIMESTEP_HZ: f64 = 64.0;
const SERVER_REPLICATION_INTERVAL: Duration = Duration::from_millis(100);

const SERVER_PORT: u16 = 5555;
const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), SERVER_PORT);

fn get_client_id() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

struct NetworkPlugin {
    pub host: bool,
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ClientPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        });
        if self.host {
            app.add_plugins(ServerPlugins {
                tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
            });
        }

        app.add_plugins(ProtocolPlugin);

        app.add_plugins(ClientPlugin);
        if self.host {
            app.add_plugins(ServerPlugin);
        }

        if self.host {
            app.add_systems(Startup, setup_host_connections);
        } else {
            app.add_systems(Startup, setup_client_connection);
        }
    }
}

fn setup_host_connections(mut commands: Commands) {
    let server = commands
        .spawn((Name::new("ServerListener"), ServerListener))
        .id();

    commands.spawn((Name::new("ClientConnection"), HostConnection { server }));
}

fn setup_client_connection(mut commands: Commands) {
    commands.spawn((
        Name::new("ClientConnection"),
        ClientConnection {
            address: SERVER_ADDR,
        },
    ));
}

#[derive(Debug, Clone, Component, Reflect)]
struct ClientConnection {
    address: SocketAddr,
}

#[derive(Debug, Clone, Component, Reflect)]
struct HostConnection {
    server: Entity,
}

struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ClientConnection>()
            .register_type::<HostConnection>();

        app.add_observer(on_client_connection_added);
        app.add_observer(on_host_connection_added);
        app.add_systems(Update, process_event_from_server);
    }
}

fn on_client_connection_added(
    trigger: Trigger<OnAdd, ClientConnection>,
    q_connection: Query<&ClientConnection, Added<ClientConnection>>,
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
        ))
        .id();

    commands.trigger_targets(Connect, client);

    Ok(())
}

fn process_event_from_server(
    mut ev_server: EventReader<ServerExampleEvent>,
    mut ev_client: EventWriter<ClientExampleEvent>,
) {
    for event in ev_server.read() {
        info!("Received event from server: {:?}", event);

        ev_client.write(ClientExampleEvent {
            message: format!("Echo: {}", event.message),
        });
    }
}

#[derive(Debug, Clone, Component)]
struct ServerListener;

struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_server_listener_added);
        app.add_observer(on_new_client);
        app.add_observer(on_new_connection);

        app.add_systems(Update, process_event_from_client);
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
    q_connected: Query<(&RemoteId, Has<HostClient>), With<ClientOf>>,
    mut ev_server: EventWriter<ToClient<ServerExampleEvent>>,
    mut ev_host: EventWriter<ServerExampleEvent>,
    _: Single<&Server>,
) -> Result {
    info!("New connection established: {:?}", trigger.target());

    let entity = trigger.target();
    let (RemoteId(peer), is_host) = q_connected.get(entity)?;
    if is_host {
        debug!("This is a host client connection");

        ev_host.write(ServerExampleEvent {
            message: "Welcome to the host server!".to_string(),
        });
    } else {
        ev_server.write(ToClient {
            target: NetworkTarget::Single(*peer),
            event: ServerExampleEvent {
                message: "Welcome to the server!".to_string(),
            },
        });
    }

    Ok(())
}

fn process_event_from_client(
    mut ev_client: EventReader<FromClient<ClientExampleEvent>>,
    _: Single<&Server>,
) {
    for event in ev_client.read() {
        info!("Received event from client: {:?}", event);
    }
}

#[derive(Debug, Clone, Event, Serialize, Deserialize)]
struct ClientExampleEvent {
    pub message: String,
}

#[derive(Debug, Clone, Event, Serialize, Deserialize)]
struct ServerExampleEvent {
    pub message: String,
}

struct MessageChannel;

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_server_event::<ServerExampleEvent, MessageChannel>();
        app.add_client_event::<ClientExampleEvent, MessageChannel>();

        app.add_channel::<MessageChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::Bidirectional);
    }
}
