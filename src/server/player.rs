//! The player plugin handles the server side player logic.

use crate::protocol::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;

pub(crate) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, handle_spawn_player);
    }
}

fn handle_spawn_player(
    mut commands: Commands,
    mut q_receiver: Query<(Entity, &RemoteId, &mut MessageReceiver<ClientSpawnRequest>)>,
    q_player: Query<(Entity, &PlayerId), With<PlayerCharacter>>,
) {
    for (entity, RemoteId(peer), mut receiver) in q_receiver.iter_mut() {
        for _ in receiver.receive() {
            if q_player.iter().any(|(_, id)| id.0 == *peer) {
                warn!(
                    "Player with ID {:?} already has a character, ignoring spawn request",
                    peer
                );
                continue;
            }

            debug!("Spawning player character for peer {:?}", peer);

            commands.spawn((
                PlayerId(*peer),
                Name::new("Player"),
                ActionState::<CharacterAction>::default(),
                Position(Vec3::new(0.0, 3.0, 0.0)), // TODO: Make sure the position is valid
                Replicate::to_clients(NetworkTarget::All),
                PredictionTarget::to_clients(NetworkTarget::All),
                ControlledBy {
                    owner: entity,
                    lifetime: Lifetime::default(),
                },
                PlayerCharacter,
            ));
        }
    }
}
