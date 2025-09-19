use avian3d::position::{PreSolveAccumulatedTranslation, PreSolveRotation, PreviousRotation};
use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::connection::host::HostClient;
use lightyear::input::{config::InputConfig, leafwing::prelude::*};
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub mod prelude {
    pub use super::*;
}

pub const PROTOCOL_ID: u64 = 0;

pub const FIXED_TIMESTEP_HZ: f64 = 64.0;

pub const SERVER_REPLICATION_INTERVAL: Duration = Duration::from_millis(100);

pub const SERVER_PORT: u16 = 5555;
pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), SERVER_PORT);

// --- Input ---

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
pub enum CharacterAction {
    #[actionlike(DualAxis)]
    Move,
    Jump,
    #[actionlike(DualAxis)]
    Look,
}

// --- Components ---

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Reflect)]
pub struct PlayerId(pub PeerId);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct PlayerName(pub String);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct PlayerMetadata;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct PlayerCharacter;

// --- Messages ---

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ServerWelcomeMessage;

#[derive(Debug, Clone, Event, Deref, DerefMut)]
pub struct ServerWelcomeEvent(pub PeerId);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClientMetaMessage {
    pub username: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClientSpawnRequest;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClientChatMessage {
    // NOTE: we probably need to also add the peer id or something to identify the sender
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ServerChatMessage {
    // NOTE: we probably want to add some metadata like sender id, timestamp, etc.
    pub sender: PeerId,
    pub message: String,
}

#[derive(Debug, Clone, Event)]
pub struct ServerChatMessageEvent {
    pub sender: PeerId,
    pub message: String,
}

pub struct MessageChannel;

// --- Plugin ---

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // Input handling
        app.add_plugins(InputPlugin::<CharacterAction> {
            config: InputConfig::<CharacterAction> {
                rebroadcast_inputs: true,
                ..default()
            },
        });

        // Components for player
        app.register_type::<PlayerId>()
            .register_type::<PlayerName>()
            .register_type::<PlayerCharacter>()
            .register_type::<PlayerMetadata>();

        app.register_component::<Name>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<PlayerId>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<PlayerName>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<PlayerMetadata>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<PlayerCharacter>()
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

        app.register_component::<ComputedAngularInertia>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<ComputedCenterOfMass>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<AccumulatedTranslation>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<PreSolveAccumulatedTranslation>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<PreSolveRotation>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<PreviousRotation>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<Friction>()
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

        // Messages and channels
        app.add_message::<ServerWelcomeMessage>()
            .add_direction(NetworkDirection::ServerToClient);
        app.add_event::<ServerWelcomeEvent>();

        app.add_message::<ClientMetaMessage>()
            .add_direction(NetworkDirection::ClientToServer);
        app.add_message::<ClientSpawnRequest>()
            .add_direction(NetworkDirection::ClientToServer);
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

        app.add_systems(FixedUpdate, on_server_chat_message);
        app.add_systems(FixedUpdate, on_client_chat_message);
    }
}

pub fn get_client_id() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

fn position_should_rollback(this: &Position, that: &Position) -> bool {
    (this.0 - that.0).length() >= f32::EPSILON
}

fn rotation_should_rollback(this: &Rotation, that: &Rotation) -> bool {
    this.angle_between(that.0) >= f32::EPSILON
}

fn on_server_welcome(
    mut ev_welcome: EventReader<ServerWelcomeEvent>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
) -> Result {
    for ev in ev_welcome.read() {
        let peer = **ev;
        sender.send::<_, MessageChannel>(
            &ServerWelcomeMessage,
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

    for _ in receiver.receive() {
        ev_welcome.write(ServerWelcomeEvent(*peer));
    }
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

fn on_client_chat_message(
    mut q_receiver: Query<&mut MessageReceiver<ServerChatMessage>>,
    mut ev_chat: EventWriter<ServerChatMessageEvent>,
) {
    for mut receiver in q_receiver.iter_mut() {
        for message in receiver.receive() {
            ev_chat.write(ServerChatMessageEvent {
                sender: message.sender.clone(),
                message: message.message.clone(),
            });
        }
    }
}
