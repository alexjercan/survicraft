use crate::helpers::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
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

pub struct MessageChannel;

// --- Plugin ---

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // Input handling
        app.add_plugins(InputPlugin {
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

        // Messages and channels
        app.add_message::<ServerWelcomeMessage>()
            .add_direction(NetworkDirection::ServerToClient);
        app.add_message::<ClientMetaMessage>()
            .add_direction(NetworkDirection::ClientToServer);
        app.add_message::<ClientSpawnRequest>()
            .add_direction(NetworkDirection::ClientToServer);
        app.add_message::<ClientChatMessage>()
            .add_direction(NetworkDirection::ClientToServer);
        app.add_message::<ServerChatMessage>()
            .add_direction(NetworkDirection::ServerToClient);

        app.add_channel::<MessageChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::Bidirectional);

        app.add_observer(add_player_character);
        app.add_systems(Update, update_player_character);

        app.add_systems(Update, testing_query);
    }
}

pub fn get_client_id() -> u64 {
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

fn add_player_character(
    trigger: Trigger<OnAdd, PlayerCharacter>,
    q_player: Query<Entity, (With<PlayerCharacter>, Without<Replicated>)>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    if !q_player.contains(entity) {
        return;
    }

    commands.entity(entity).insert(PlayerCharacterController);
}

fn update_player_character(
    mut q_player: Query<
        (&mut PlayerCharacterInput, &ActionState<CharacterAction>),
        With<PlayerCharacterController>,
    >,
) {
    for (mut input, action_state) in q_player.iter_mut() {
        input.move_axis = action_state.axis_pair(&CharacterAction::Move);
        input.jump = action_state.just_pressed(&CharacterAction::Jump);
    }
}

fn testing_query(
    q_confirmed: Query<(Entity, &Confirmed) /*, Without<LeafwingSequence<CharacterAction>>>, */ >,
) {
    for (entity, confirmed) in q_confirmed.iter() {
        info!("Entity {:?} is confirmed: {:?}", entity, confirmed);
    }
}
