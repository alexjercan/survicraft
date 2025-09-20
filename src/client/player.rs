//! The player plugin handles the server side player logic.

use crate::protocol::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::*;

/// Event that is used to signal that a player want's to be spawned.
#[derive(Debug, Clone, Event)]
pub struct ClientSpawnPlayerEvent;

pub(crate) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ClientSpawnPlayerEvent>();
        app.add_systems(Update, on_spawn_player);
    }
}

fn on_spawn_player(
    mut ev_spawn: EventReader<ClientSpawnPlayerEvent>,
    sender: Single<(&RemoteId, &mut MessageSender<ClientSpawnRequest>)>,
) {
    let (RemoteId(peer), mut sender) = sender.into_inner();

    for _ in ev_spawn.read() {
        debug!("Sending spawn request for player {:?}", peer);

        sender.send::<MessageChannel>(ClientSpawnRequest);
    }
}
