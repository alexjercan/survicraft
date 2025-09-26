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

use super::{controller::*, resources::*, states::*};
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
    mut commands: Commands,
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

        match local {
            PeerId::Local(_) => {
                // NOTE: I don't like this way of handling things but it is what it is.
                // Host cannot send messages to server because it doesn't have a transport layer...
                debug!(
                    "Host spawn player metadata for peer {:?}: {:?}",
                    peer, player_name
                );

                commands.spawn((
                    Name::new("PlayerMetadata"),
                    PlayerId(peer),
                    PlayerMetadata {
                        username: player_name.to_string(),
                    },
                    Replicate::to_clients(NetworkTarget::All),
                ));
            }
            _ => {
                let metadata = ClientMetaMessage {
                    username: player_name.to_string(),
                };
                debug!("Sending client metadata: {:?}", metadata);
                sender.send::<MessageChannel>(metadata);
            }
        }
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
            .register_type::<PlayerMetadata>()
            .register_type::<PlayerController>()
            .register_type::<HeadControllerMarker>();

        // Register components for replication

        app.register_component::<Name>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<RigidBody>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<PlayerId>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<PlayerMetadata>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<PlayerController>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<HeadControllerMarker>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        // Fully replicated, but not visual, so no need for lerp/corrections:
        app.register_component::<LinearVelocity>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<AngularVelocity>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<ExternalForce>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<ExternalTorque>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<ExternalImpulse>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<ExternalAngularImpulse>()
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
        app.world_mut()
            .resource_mut::<InterpolationRegistry>()
            .set_interpolation::<Transform>(TransformLinearInterpolation::lerp);
        app.world_mut()
            .resource_mut::<InterpolationRegistry>()
            .set_interpolation_mode::<Transform>(InterpolationMode::None);

        // Messages and channels
        app.add_message::<ServerWelcomeMessage>()
            .add_direction(NetworkDirection::ServerToClient);
        app.add_event::<ServerWelcomeEvent>();

        app.add_message::<ClientMetaMessage>()
            .add_direction(NetworkDirection::ClientToServer);

        app.add_message::<ClientSpawnRequest>()
            .add_direction(NetworkDirection::ClientToServer);
        app.add_event::<ClientSpawnPlayerEvent>();
        app.add_event::<ServerSpawnPlayerEvent>();

        app.add_message::<ClientChatMessage>()
            .add_direction(NetworkDirection::ClientToServer);

        app.add_message::<ServerChatMessage>()
            .add_direction(NetworkDirection::ServerToClient);
        app.add_event::<ServerChatMessageEvent>();

        app.add_channel::<MessageChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::Bidirectional);

        // Systems
        app.add_systems(FixedUpdate, on_server_welcome);
        app.add_systems(FixedUpdate, on_client_welcome);

        app.add_systems(Update, on_client_spawn_event);
        app.add_systems(Update, on_server_spawn_request);
        app.add_systems(Update, on_server_spawn_player);

        app.add_systems(FixedUpdate, on_server_chat_message);
        app.add_systems(Update, on_trigger_chat_message);
        app.add_systems(FixedUpdate, receive_server_chat_message);
        app.add_systems(FixedUpdate, receive_client_chat_message);
        app.add_systems(Update, on_chat_message_submit);
    }
}

/// ServerWelcomeEvent
///
/// Flow:
/// - Server detects new connection (on_new_connection)
/// - Server triggers ServerWelcomeEvent with peer ID and world seed
/// - on_server_welcome system sends ServerWelcomeMessage to the client
/// - Client receives ServerWelcomeMessage in on_client_welcome system
/// - Client triggers ServerWelcomeEvent with peer ID and world seed
/// - Client can now use the world seed for terrain generation
///
/// This will work for both dedicated servers and host clients.
#[derive(Debug, Clone, Event)]
struct ServerWelcomeEvent {
    pub peer: PeerId,
    pub seed: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct ServerWelcomeMessage {
    pub seed: u32,
}

fn on_server_welcome(
    mut ev_welcome: EventReader<ServerWelcomeEvent>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
) -> Result {
    for &ServerWelcomeEvent { peer, seed } in ev_welcome.read() {
        sender.send::<_, MessageChannel>(
            &ServerWelcomeMessage { seed },
            server.clone(),
            &NetworkTarget::Single(peer),
        )?;
    }

    Ok(())
}

fn on_client_welcome(
    receiver: Single<
        (&RemoteId, &mut MessageReceiver<ServerWelcomeMessage>),
        Or<(With<Client>, With<HostClient>)>,
    >,
    mut ev_welcome: EventWriter<ServerWelcomeEvent>,
) {
    let (RemoteId(peer), mut receiver) = receiver.into_inner();

    for ServerWelcomeMessage { seed } in receiver.receive() {
        ev_welcome.write(ServerWelcomeEvent { peer: *peer, seed });
    }
}

/// The ClientMetaMessage is sent from the client to the server after receiving the
/// ServerWelcomeMessage. It contains metadata about the client, such as the username.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct ClientMetaMessage {
    pub username: String,
}

/// Event that is used to signal that a player wants to be spawned.
///
/// When this event is triggered, the client will send a ClientSpawnRequest message to the server,
/// which will then spawn the player and replicate it to all clients.
///
/// Flow:
/// - Client triggers ClientSpawnPlayerEvent (e.g. when entering the Playing state)
/// - on_client_spawn_event system sends ClientSpawnRequest to the server
/// - Server receives ClientSpawnRequest in on_server_spawn_request system
/// - Server triggers ServerSpawnPlayerEvent with the peer ID
/// - Server handles ServerSpawnPlayerEvent to actually spawn the player entity
#[derive(Debug, Clone, Event)]
pub(crate) struct ClientSpawnPlayerEvent;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct ClientSpawnRequest;

#[derive(Debug, Clone, Event)]
pub(crate) struct ServerSpawnPlayerEvent {
    pub owner: Entity,
    pub peer: PeerId,
}

fn on_client_spawn_event(
    mut ev_spawn: EventReader<ClientSpawnPlayerEvent>,
    sender: Single<(&RemoteId, &mut MessageSender<ClientSpawnRequest>)>,
) {
    let (RemoteId(peer), mut sender) = sender.into_inner();

    for _ in ev_spawn.read() {
        debug!("Sending spawn request for player {:?}", *peer);

        sender.send::<MessageChannel>(ClientSpawnRequest);
    }
}

fn on_server_spawn_request(
    mut q_receiver: Query<(Entity, &RemoteId, &mut MessageReceiver<ClientSpawnRequest>)>,
    mut ev_spawn: EventWriter<ServerSpawnPlayerEvent>,
    _: Single<&Server>,
) {
    for (entity, RemoteId(peer), mut receiver) in q_receiver.iter_mut() {
        for _ in receiver.receive() {
            debug!("Received spawn request for player {:?}", peer);

            ev_spawn.write(ServerSpawnPlayerEvent {
                owner: entity,
                peer: *peer,
            });
        }
    }
}

#[derive(Component, Debug, Clone)]
pub(super) struct NetworkPlayerController {
    pub owner: Entity,
    pub peer: PeerId,
}

fn on_server_spawn_player(
    mut commands: Commands,
    q_player: Query<(Entity, &NetworkPlayerController), Added<NetworkPlayerController>>,
) {
    for (entity, NetworkPlayerController { owner, peer }) in &q_player {
        debug!("Adding PlayerId and ControlledBy to entity {entity:?}");

        commands.entity(entity).insert((
            PlayerId(*peer),
            Replicate::to_clients(NetworkTarget::All),
            PredictionTarget::to_clients(NetworkTarget::All),
            ControlledBy {
                owner: *owner,
                lifetime: Lifetime::default(),
            },
        ));
    }
}

/// Event that is used to signal that a chat message has been submitted by a player.
///
/// Flow:
/// - Player submits chat message (e.g. via UI) and triggers ChatMessageSubmittedEvent
/// - on_chat_message_submit is triggered
/// - on_chat_message_submit system sends ClientChatMessage to the server
/// - Server receives ClientChatMessage in receive_client_chat_message system
/// - Server triggers ServerChatMessageEvent with the peer ID and message
/// - on_server_chat_message system sends ServerChatMessage to all clients
/// - on_server_chat_message system also logs the message and triggers AddChatHistoryItemEvent
/// - Clients receive ServerChatMessage in receive_server_chat_message system
/// - Clients trigger ServerChatMessageEvent with the peer ID and message
///
/// This will work for both dedicated servers and host clients.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct ServerChatMessage {
    // NOTE: we probably want to add some metadata like sender id, timestamp, etc.
    pub sender: PeerId,
    pub message: String,
}

#[derive(Debug, Clone, Event)]
struct ServerChatMessageEvent {
    pub sender: PeerId,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct ClientChatMessage {
    // NOTE: we probably need to also add the peer id or something to identify the sender
    pub message: String,
}

fn on_server_chat_message(
    mut ev_chat: EventReader<ServerChatMessageEvent>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
) -> Result {
    for ev in ev_chat.read() {
        sender.send::<_, MessageChannel>(
            &ServerChatMessage {
                sender: ev.sender,
                message: ev.message.clone(),
            },
            server.clone(),
            &NetworkTarget::All,
        )?;
    }

    Ok(())
}

fn on_trigger_chat_message(
    mut ev_chat: EventReader<ServerChatMessageEvent>,
    q_players: Query<(&PlayerMetadata, &PlayerId)>,
    mut ev_history: EventWriter<AddChatHistoryItemEvent>,
) -> Result {
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
            error!(
                "Received chat message from unknown player ID {:?}",
                ev.sender
            );
        }
    }

    Ok(())
}

fn receive_server_chat_message(
    mut q_receiver: Query<&mut MessageReceiver<ServerChatMessage>>,
    mut ev_chat: EventWriter<ServerChatMessageEvent>,
) {
    for mut receiver in q_receiver.iter_mut() {
        for message in receiver.receive() {
            debug!("Received chat message from {:?}: {}", message.sender, message.message);

            ev_chat.write(ServerChatMessageEvent {
                sender: message.sender,
                message: message.message.clone(),
            });
        }
    }
}

fn receive_client_chat_message(
    mut q_receiver: Query<(&RemoteId, &mut MessageReceiver<ClientChatMessage>)>,
    mut ev_chat: EventWriter<ServerChatMessageEvent>,
) {
    for (RemoteId(peer), mut receiver) in q_receiver.iter_mut() {
        for message in receiver.receive() {
            debug!("Received chat message from {:?}: {}", peer, message.message);

            ev_chat.write(ServerChatMessageEvent {
                sender: *peer,
                message: message.message.clone(),
            });
        }
    }
}

fn on_chat_message_submit(
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

struct MessageChannel;
