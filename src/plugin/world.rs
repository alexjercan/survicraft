use avian3d::prelude::*;
use bevy::prelude::*;

use super::{controller::*, states::*};
use crate::common::prelude::*;

const DISCOVER_RADIUS: u32 = 5;
const INITIAL_TERRAIN_RADIUS: u32 = 5;

pub(super) struct WorldGenerationPlugin {
    pub render: bool,
}

impl Plugin for WorldGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TerrainPlugin::default());
        app.add_plugins(FeaturesPlugin::default());

        if self.render {
            app.add_plugins(TerrainRenderPlugin::default());
            app.add_plugins(FeaturesRenderPlugin::default());
        }

        app.add_systems(OnEnter(LauncherStates::Generating), setup_initial_terrain);
        app.add_systems(Update, discover_terrain_at_player_position);
    }
}

fn setup_initial_terrain(mut ev_discover: EventWriter<TileDiscoverEvent>) {
    ev_discover.write(TileDiscoverEvent::new(Vec2::ZERO, INITIAL_TERRAIN_RADIUS));
}

fn discover_terrain_at_player_position(
    q_player: Query<&Position, With<PlayerControllerMarker>>,
    mut ev_discover: EventWriter<TileDiscoverEvent>,
) {
    for player_pos in q_player.iter() {
        let tile_pos = Vec2::new(player_pos.x, player_pos.z);
        ev_discover.write(TileDiscoverEvent::new(tile_pos, DISCOVER_RADIUS));
    }
}
