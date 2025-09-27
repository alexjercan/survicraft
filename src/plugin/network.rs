use std::{
    fmt::Debug,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use avian3d::prelude::*;
use bevy::{prelude::*, reflect::GetTypeRegistration};
use leafwing_input_manager::prelude::*;
use lightyear::{
    connection::host::HostClient,
    input::config::InputConfig,
    netcode::{Key, NetcodeClient, NetcodeServer},
    prelude::{
        client::ClientPlugins,
        input::leafwing::InputPlugin,
        server::{ClientOf, ServerPlugins, ServerUdpIo, Start},
        *,
    },
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::{resources::*, states::*};
use crate::prelude::*;

// --- Contants ---

pub(super) const PROTOCOL_ID: u64 = 0;

pub(super) const FIXED_TIMESTEP_HZ: f64 = 64.0;

pub(super) const SERVER_REPLICATION_INTERVAL: Duration = Duration::from_millis(100);

pub(super) const SERVER_PORT: u16 = 5555;
pub(super) const SERVER_ADDR: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), SERVER_PORT);

/// Main Network Plugin
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
        app.add_plugins(WelcomePlugin);
        app.add_plugins(ControllerPlugin {
            dedicated: self.dedicated,
        });
        app.add_plugins(ChatPlugin);

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

/// Component to track if the client is ready (i.e. has received the welcome message from the
/// server).
#[derive(Resource, Debug, Clone, PartialEq, Eq, Deref, DerefMut, Reflect)]
pub(super) struct ClientNetworkStateReady(pub bool);

/// Structure representing a request to connect to a server.
/// To connect to the server, add this component to an entity.
#[derive(Debug, Clone, Component, Reflect)]
struct ClientConnection {
    address: SocketAddr,
}

/// Structure representing a request to connect to the host server.
/// To connect to the host server, add this component to an entity.
#[derive(Debug, Clone, Component, Reflect)]
struct HostConnection {
    server: Entity,
}

struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ClientConnection>()
            .register_type::<HostConnection>();

        app.insert_resource(ClientNetworkStateReady(false));

        app.add_observer(on_client_connection_added);
        app.add_observer(on_host_connection_added);
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

/// Server component to mark the server entity.
/// Add this component to an entity to make it a server.
#[derive(Debug, Clone, Component)]
struct ServerListener;

struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_server_listener_added);
        app.add_observer(on_new_client);
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
    debug!("New client connected: {:?}", trigger.target());

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

/// Wrapper around the Leafwing InputManagerPlugin to configure it for networked use.
pub(super) struct NetworkInputPlugin<A> {
    pub rebroadcast_inputs: bool,
    _marker: std::marker::PhantomData<A>,
}

impl<A> Default for NetworkInputPlugin<A> {
    fn default() -> Self {
        Self {
            rebroadcast_inputs: true,
            _marker: std::marker::PhantomData,
        }
    }
}

/// NOTE: This is already defined in lightyear, but it is not public, so I need to redefine it
/// here.
trait LeafwingUserAction:
    Serialize
    + DeserializeOwned
    + Clone
    + PartialEq
    + Send
    + Sync
    + Debug
    + 'static
    + Copy
    + Actionlike
    + GetTypeRegistration
{
}

impl<
        A: Serialize
            + DeserializeOwned
            + Clone
            + PartialEq
            + Send
            + Sync
            + Debug
            + 'static
            + Copy
            + Actionlike
            + GetTypeRegistration,
    > LeafwingUserAction for A
{
}

impl<A: LeafwingUserAction> Plugin for NetworkInputPlugin<A> {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputPlugin::<A> {
            config: InputConfig::<A> {
                rebroadcast_inputs: self.rebroadcast_inputs,
                ..default()
            },
        });
    }
}

// --- Utility Functions ---

fn get_client_id() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

fn position_should_rollback(this: &Position, that: &Position) -> bool {
    (this.0 - that.0).length() >= 0.01
}

fn rotation_should_rollback(this: &Rotation, that: &Rotation) -> bool {
    this.angle_between(that.0) >= 0.01
}

/// Component used to identify which player "owns" an entity.
/// This can be added on player characters, player-placed objects, etc.
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Reflect)]
pub(super) struct PlayerId(pub PeerId);

/// Component used to store metadata about a player, such as their username.
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub(super) struct PlayerMetadata {
    pub username: String,
}

/// Marker component for the head entity, which is responsible for character rotation.
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub(super) struct HeadControllerMarker;

/// Protocol Plugin
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // Components for player
        app.register_type::<PlayerId>()
            .register_type::<PlayerMetadata>();

        // Register components for replication
        app.register_component::<Name>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<PlayerId>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<PlayerMetadata>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        // Fully replicated, but not visual, so no need for lerp/corrections:
        app.register_component::<LinearVelocity>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<AngularVelocity>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<ExternalForce>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<ExternalImpulse>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<ComputedMass>()
            .add_prediction(PredictionMode::Full);

        // Position and Rotation have a `correction_fn` set, which is used to smear rollback errors
        // over a few frames, just for the rendering part in postudpate.
        //
        // They also set `interpolation_fn` which is used by the VisualInterpolationPlugin to smooth
        // out rendering between fixedupdate ticks.
        app.register_component::<Position>()
            .add_prediction(PredictionMode::Full)
            .add_should_rollback(position_should_rollback)
            .add_linear_correction_fn()
            .add_interpolation(InterpolationMode::Full)
            .add_linear_interpolation_fn();

        app.register_component::<Rotation>()
            .add_prediction(PredictionMode::Full)
            .add_should_rollback(rotation_should_rollback)
            .add_linear_correction_fn()
            .add_interpolation(InterpolationMode::Full)
            .add_linear_interpolation_fn();

        // Do not replicate Transform but register interpolation for visual interpolation
        app.register_component::<Transform>()
            .add_prediction(PredictionMode::None);
        app.world_mut()
            .resource_mut::<InterpolationRegistry>()
            .set_interpolation::<Transform>(TransformLinearInterpolation::lerp);
        app.world_mut()
            .resource_mut::<InterpolationRegistry>()
            .set_interpolation_mode::<Transform>(InterpolationMode::None);
    }
}

// --- Welcome Plugin ---
//
// This plugin handles the initial welcome message sent from the server to the client
// when it connects. The welcome message contains important information such as the
// world seed.
//
// Then the client responds with its metadata (e.g. username) as an ACK to the server.

#[derive(Debug, Clone, Event, Serialize, Deserialize)]
struct ServerWelcomeEvent {
    pub seed: u32,
}

#[derive(Debug, Clone, Event, Serialize, Deserialize)]
struct ClientMetadataEvent {
    pub username: String,
}

struct WelcomeChannel;

pub struct WelcomePlugin;

impl Plugin for WelcomePlugin {
    fn build(&self, app: &mut App) {
        app.add_server_event::<ServerWelcomeEvent, WelcomeChannel>();
        app.add_client_event::<ClientMetadataEvent, WelcomeChannel>();

        app.add_observer(on_new_connection);
        app.add_systems(Update, on_server_welcome_message);
        app.add_systems(Update, on_client_metadata_message);

        app.add_channel::<WelcomeChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::Bidirectional);
    }
}

fn on_new_connection(
    trigger: Trigger<OnAdd, Connected>,
    q_connected: Query<(&RemoteId, Has<HostClient>), With<ClientOf>>,
    mut ev_server: EventWriter<ToClient<ServerWelcomeEvent>>,
    mut ev_host: EventWriter<ServerWelcomeEvent>,
    world_seed: Res<TerrainGenerationSeed>,
    _: Single<&Server>,
) -> Result {
    debug!("New connection established: {:?}", trigger.target());

    let entity = trigger.target();
    let (RemoteId(peer), is_host) = q_connected.get(entity)?;
    if is_host {
        debug!("This is a host client connection");

        ev_host.write(ServerWelcomeEvent { seed: **world_seed });
    } else {
        ev_server.write(ToClient {
            target: NetworkTarget::Single(*peer),
            event: ServerWelcomeEvent { seed: **world_seed },
        });
    }

    Ok(())
}

fn on_server_welcome_message(
    mut ev_welcome: EventReader<ServerWelcomeEvent>,
    mut ev_metadata: EventWriter<ClientMetadataEvent>,
    player_name: Res<PlayerNameSetting>,
    mut client_ready: ResMut<ClientNetworkStateReady>,
    mut world_seed: ResMut<TerrainGenerationSeed>,
) {
    for &ServerWelcomeEvent { seed } in ev_welcome.read() {
        debug!("Received welcome message from server");
        **client_ready = true;
        **world_seed = seed;

        ev_metadata.write(ClientMetadataEvent {
            username: player_name.to_string(),
        });
    }
}

fn on_client_metadata_message(
    mut commands: Commands,
    mut ev_metadata: EventReader<FromClient<ClientMetadataEvent>>,
) {
    for FromClient { peer, event, .. } in ev_metadata.read() {
        debug!("Spawn player metadata for peer {:?}: {:?}", peer, event);

        commands.spawn((
            Name::new("PlayerMetadata"),
            PlayerId(*peer),
            PlayerMetadata {
                username: event.username.clone(),
            },
            Replicate::to_clients(NetworkTarget::All),
        ));
    }
}

// --- Controller Plugin ---
//
// When we enter the Playing state we need to send a spawn request to the server.

#[derive(Debug, Clone, Event, Serialize, Deserialize)]
pub struct ClientSpawnPlayerEvent;

struct SpawnChannel;

pub struct ControllerPlugin {
    dedicated: bool,
}

impl Plugin for ControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_client_event::<ClientSpawnPlayerEvent, SpawnChannel>();

        if !self.dedicated {
            app.add_systems(OnEnter(LauncherStates::Playing), send_spawn_request);
        }

        app.add_channel::<SpawnChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::Bidirectional);
    }
}

fn send_spawn_request(mut ev_spawn: EventWriter<ClientSpawnPlayerEvent>) {
    debug!("Sending spawn request to server");
    ev_spawn.write(ClientSpawnPlayerEvent);
}

// --- Chat Plugin ---
//
// This plugin handles chat messages sent from clients to the server and broadcasted to all
// clients.

#[derive(Debug, Clone, Event, Serialize, Deserialize)]
pub struct ClientChatMessageEvent {
    pub message: String,
}

#[derive(Debug, Clone, Event, Serialize, Deserialize)]
pub struct ServerChatMessageEvent {
    pub sender: PeerId,
    pub message: String,
}

struct MessageChannel;

pub struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_client_event::<ClientChatMessageEvent, MessageChannel>();
        app.add_server_event::<ServerChatMessageEvent, MessageChannel>();

        app.add_systems(
            Update,
            (
                on_chat_message_submit,
                on_server_chat_message,
                on_client_chat_message,
            )
                .chain(),
        );

        app.add_channel::<MessageChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::Bidirectional);
    }
}

fn on_chat_message_submit(
    mut ev_submitted: EventReader<ChatMessageSubmittedEvent>,
    mut ev_client: EventWriter<ClientChatMessageEvent>,
) {
    for ev in ev_submitted.read() {
        let msg = ev.message.trim();
        debug!("Player submitted chat message: {}", msg);

        if !msg.is_empty() {
            ev_client.write(ClientChatMessageEvent {
                message: msg.to_string(),
            });
        }
    }
}

fn on_server_chat_message(
    mut ev_client: EventReader<FromClient<ClientChatMessageEvent>>,
    mut ev_server: EventWriter<ToClient<ServerChatMessageEvent>>,
    mut ev_host: EventWriter<ServerChatMessageEvent>,
) -> Result {
    for FromClient { peer, event, .. } in ev_client.read() {
        debug!("Received chat message from {:?}: {}", peer, event.message);

        ev_server.write(ToClient {
            target: NetworkTarget::All,
            event: ServerChatMessageEvent {
                sender: *peer,
                message: event.message.clone(),
            },
        });

        // NOTE: if this is a host client, also write the event locally
        ev_host.write(ServerChatMessageEvent {
            sender: *peer,
            message: event.message.clone(),
        });
    }

    Ok(())
}

fn on_client_chat_message(
    mut ev_client: EventReader<ServerChatMessageEvent>,
    mut ev_history: EventWriter<AddChatHistoryItemEvent>,
    q_players: Query<(&PlayerMetadata, &PlayerId)>,
) {
    for ev in ev_client.read() {
        if let Some((PlayerMetadata { username, .. }, _)) =
            q_players.iter().find(|(_, id)| id.0 == ev.sender)
        {
            debug!("Received chat message from {}: {}", username, ev.message);

            ev_history.write(AddChatHistoryItemEvent {
                sender: username.clone(),
                message: ev.message.clone(),
            });
        } else {
            error!(
                "Received chat message from unknown player ID {:?}",
                ev.sender
            );
        }
    }
}
