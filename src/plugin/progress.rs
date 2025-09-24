use bevy::prelude::*;
use iyes_progress::prelude::*;

use super::{network::*, states::*};
use crate::prelude::*;

pub(super) struct LauncherProgressPlugin;

impl Plugin for LauncherProgressPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            ProgressPlugin::<LauncherStates>::new()
                .with_state_transition(LauncherStates::Connecting, LauncherStates::Generating)
                .with_state_transition(LauncherStates::Generating, LauncherStates::Playing),
        );

        app.add_systems(
            Update,
            check_connection_progress
                .track_progress::<LauncherStates>()
                .run_if(in_state(LauncherStates::Connecting)),
        );

        app.add_systems(
            Update,
            check_terrain_generation_progress
                .track_progress::<LauncherStates>()
                .run_if(in_state(LauncherStates::Generating)),
        );

        app.add_systems(OnEnter(LauncherStates::Playing), || {
            info!("Entered Playing state...");
        });
    }
}

fn check_connection_progress(client_ready: Option<Res<ClientNetworkStateReady>>) -> Progress {
    match client_ready {
        None => {
            debug!("Connection progress: 1/1 (dedicated server)");

            Progress { done: 1, total: 1 }
        }
        Some(client_ready) => {
            let done = if **client_ready { 1 } else { 0 };
            debug!("Connection progress: {}/1", done);

            Progress { done, total: 1 }
        }
    }
}

fn check_terrain_generation_progress(terrain_progress: Res<TerrainGenerationProgress>) -> Progress {
    let total = terrain_progress.total_chunks.max(1); // Avoid division by zero
    debug!(
        "Terrain generation progress: {}/{} chunks",
        terrain_progress.generated_chunks, total
    );
    Progress {
        done: terrain_progress.generated_chunks,
        total: total,
    }
}
