//! The player plugin handles the server side player logic.

use crate::helpers::prelude::*;
use crate::protocol::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PlayerPluginSet;

pub(crate) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_spawn_player, handle_player_actions).in_set(PlayerPluginSet),
        );
    }
}

fn handle_spawn_player(
    mut commands: Commands,
    mut q_receiver: Query<
        (Entity, &RemoteId, &mut MessageReceiver<ClientMetaMessage>),
        Without<Client>,
    >,
) {
    for (entity, RemoteId(peer), mut receiver) in q_receiver.iter_mut() {
        for message in receiver.receive() {
            info!("Client {:?} set their name to {}", peer, message.username);

            commands.spawn((
                PlayerId(*peer),
                PlayerName(message.username.clone()),
                Name::new("Player"),
                ActionState::<CharacterAction>::default(),
                Position(Vec3::new(0.0, 3.0, 0.0)),
                Replicate::to_clients(NetworkTarget::All),
                PredictionTarget::to_clients(NetworkTarget::All),
                ControlledBy {
                    owner: entity,
                    lifetime: Lifetime::default(),
                },
                CharacterPhysicsBundle::default(),
            ));
        }
    }
}

fn handle_player_actions(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut query: Query<(&ActionState<CharacterAction>, CharacterQuery)>,
) {
    for (action_state, mut character) in &mut query {
        apply_character_action(&time, &spatial_query, action_state, &mut character);
    }
}
