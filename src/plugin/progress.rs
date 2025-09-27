use bevy::prelude::*;
use iyes_progress::prelude::*;

use super::{network::*, states::*};
use crate::prelude::*;

#[derive(Resource, Clone, Debug, Default, Deref, DerefMut)]
pub(super) struct ProgressGeneration(pub f32);

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

        app.insert_resource(ProgressGeneration::default());
        app.add_systems(
            Update,
            check_terrain_generation_progress
                .track_progress::<LauncherStates>()
                .run_if(in_state(LauncherStates::Generating)),
        );
    }
}

fn check_connection_progress(client_ready: Option<Res<ClientNetworkStateReady>>) -> Progress {
    match client_ready {
        None => {
            trace!("Connection progress: 1/1 (dedicated server)");

            Progress { done: 1, total: 1 }
        }
        Some(client_ready) => {
            let done = if **client_ready { 1 } else { 0 };
            trace!("Connection progress: {}/1", done);

            Progress { done, total: 1 }
        }
    }
}

fn check_terrain_generation_progress(
    terrain_progress: Res<TerrainGenerationProgress>,
    mut progress: ResMut<ProgressGeneration>,
) -> Progress {
    let total = terrain_progress.total_chunks.max(1); // Avoid division by zero
    trace!(
        "Terrain generation progress: {}/{} chunks",
        terrain_progress.generated_chunks,
        total
    );
    **progress = terrain_progress.generated_chunks as f32 / total as f32;
    Progress {
        done: terrain_progress.generated_chunks,
        total,
    }
}
