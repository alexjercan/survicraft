use avian3d::prelude::*;
use crate::{helpers::prelude::*, protocol::prelude::*};
use bevy::prelude::*;

const DISCOVER_RADIUS: u32 = 5;
const INITIAL_TERRAIN_RADIUS: u32 = 5;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct InitializeTerrain;

pub struct TerrainGenerationPlugin;

impl Plugin for TerrainGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<InitializeTerrain>();

        app.add_plugins(TerrainPlugin::default());

        app.add_observer(setup_initial_terrain);
        app.add_systems(Update, discover_terrain_at_player_position);
    }
}

fn setup_initial_terrain(
    _: Trigger<OnAdd, InitializeTerrain>,
    mut ev_discover: EventWriter<TileDiscoverEvent>,
) {
    ev_discover.write(TileDiscoverEvent::new(Vec2::ZERO, INITIAL_TERRAIN_RADIUS));
}

fn discover_terrain_at_player_position(
    q_player: Query<&Position, With<PlayerCharacter>>,
    mut ev_discover: EventWriter<TileDiscoverEvent>,
) {
    for player_pos in q_player.iter() {
        let tile_pos = Vec2::new(player_pos.x, player_pos.z);
        ev_discover.write(TileDiscoverEvent::new(tile_pos, DISCOVER_RADIUS));
    }
}
