//! Square tilemap plugin for Bevy.
//!
//! This plugin provides a square tilemap system that allows for the discovery and management of
//! square tiles in a Bevy application. The tiles are organized into chunks, and the plugin
//! allows for the dynamic discovery of tiles based on a specified layout and radius.
//! The plugin uses ECS style architecture to manage square tiles, where each tile is
//! represented by an entity with a component that can be constructed from a `IVec2` coordinate.
//! The chunks are also entities that contain all the tiles within a certain radius.
//!
//! You can use the `debug` feature to enable debug visualization of the square grid.
//!
//! Tiles will be represented as
//!
//! A   B
//! +---+
//! |   |
//! +---+
//! C   D
//!
//! where each tile is a square with corners A, B, C, and D. We will anchor the tile at the center
//! of the square.

use bevy::{platform::collections::HashMap, prelude::*};
// use pathfinding::prelude::astar;

#[cfg(feature = "debug")]
use self::debug::{DebugPlugin, DebugSet};

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct TileCoord(pub IVec2);

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct LocalTileCoord(pub IVec2);

#[derive(Component, Clone, Debug, Reflect)]
pub struct TileBorder;

/// The TileDiscoverEvent is used to trigger the discovery of square tiles in the map.
/// The position is given in world coordinates, and the event is generic over a component type `C`
/// that can be constructed from a `IVec2` coordinate.
#[derive(Event, Clone, Debug)]
pub struct TileDiscoverEvent {
    /// The position in world coordinates where the discovery event occurs.
    pub pos: Vec2,
}

impl TileDiscoverEvent {
    /// Creates a new TileDiscoverEvent with the specified position.
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
        }
    }
}

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct ChunkCoord(pub IVec2);

#[derive(Resource, Debug, Clone)]
pub struct TileMapStorage {
    tile_size: Vec2,
    chunk_radius: u32,
    discover_radius: u32,
    chunks: HashMap<IVec2, Entity>,
}

impl TileMapStorage {
    fn discover_chunks(&self, center: IVec2) -> Vec<IVec2> {
        let mut chunks = Vec::new();
        let step = self.chunk_radius as i32 * 2 + 1;
        for x in -(self.discover_radius as i32)..=self.discover_radius as i32 {
            for y in -(self.discover_radius as i32)..=self.discover_radius as i32 {
                let chunk_x = center.x + x * step;
                let chunk_y = center.y + y * step;
                chunks.push(IVec2::new(chunk_x, chunk_y));
            }
        }
        chunks
    }

    fn chunk_tiles(&self, center: IVec2) -> Vec<IVec2> {
        let mut tiles = Vec::new();
        for x in -(self.chunk_radius as i32)..=(self.chunk_radius as i32) {
            for y in -(self.chunk_radius as i32)..=(self.chunk_radius as i32) {
                let tile_x = center.x + x;
                let tile_y = center.y + y;
                tiles.push(IVec2::new(tile_x, tile_y));
            }
        }
        tiles
    }

    fn chunk_border(&self, center: IVec2) -> Vec<IVec2> {
        let mut border = Vec::new();
        let r = self.chunk_radius as i32 + 1;

        for x in -r..=r {
            border.push(IVec2::new(center.x + x, center.y - r));
            border.push(IVec2::new(center.x + x, center.y + r));
        }
        for y in (-r + 1)..=(r - 1) {
            border.push(IVec2::new(center.x - r, center.y + y));
            border.push(IVec2::new(center.x + r, center.y + y));
        }
        border
    }

    fn tile_to_center(&self, tile: &IVec2) -> IVec2 {
        let step = self.chunk_radius as i32 * 2 + 1;
        ((tile + tile.signum() * self.chunk_radius as i32) / step) * step
    }

    pub fn world_pos_to_tile(&self, position: Vec2) -> IVec2 {
        IVec2::new(
            ((position.x + self.tile_size.x / 2.0) / self.tile_size.x).floor() as i32,
            ((position.y + self.tile_size.y / 2.0) / self.tile_size.y).floor() as i32,
        )
    }

    pub fn tile_to_world_pos(&self, tile: IVec2) -> Vec2 {
        Vec2::new(
            tile.x as f32 * self.tile_size.x,
            tile.y as f32 * self.tile_size.y,
        )
    }

    fn get_chunk(&self, tile: IVec2) -> Option<&Entity> {
        self.chunks.get(&tile)
    }

    fn insert_chunk(&mut self, center: IVec2, chunk: Entity) {
        self.chunks.insert(center, chunk);
    }

    // TODO: implement pathfinding
}

/// The TileMapSet is a system set used to group tile map related systems together.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TileMapSet;

/// The TileMapPlugin is a Bevy plugin that sets up the square tilemap system.
/// The plugin requires a component type `C` that can be constructed from a `IVec2` coordinate.
/// The plugin will generate a square grid based on the specified tile size, chunk radius, and
/// discover radius. It will spawn a new entity for each tile in the discovered chunks and
/// it will add the `C` component to each tile entity.
/// The tiles will be grouped into chunks, and each chunk will be represented by a `ChunkCoord`
/// component. Each tile in the chunk will be parented to the chunk entity.
pub struct TileMapPlugin {
    tile_size: Vec2,
    chunk_radius: u32,
    discover_radius: u32,
}

impl TileMapPlugin {
    pub fn new(tile_size: Vec2, chunk_radius: u32, discover_radius: u32) -> Self {
        Self {
            tile_size,
            chunk_radius,
            discover_radius,
        }
    }
}

impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ChunkCoord>()
            .register_type::<TileCoord>()
            .register_type::<LocalTileCoord>();

        #[cfg(feature = "debug")]
        app.add_plugins(DebugPlugin);
        #[cfg(feature = "debug")]
        app.configure_sets(Update, DebugSet.in_set(TileMapSet));

        app.add_event::<TileDiscoverEvent>();

        app.insert_resource(TileMapStorage {
            tile_size: self.tile_size,
            chunk_radius: self.chunk_radius,
            discover_radius: self.discover_radius,
            chunks: HashMap::default(),
        });

        app.add_systems(Update, generate_chunks.in_set(TileMapSet).chain());
    }
}

fn generate_chunks(
    mut commands: Commands,
    mut storage: ResMut<TileMapStorage>,
    mut ev_discover: EventReader<TileDiscoverEvent>,
) {
    for ev in ev_discover.read() {
        let tile = storage.world_pos_to_tile(ev.pos);
        let center = storage.tile_to_center(&tile);
        debug!("Discovering chunks around tile {:?} at center {:?}", tile, center);

        for center in storage.discover_chunks(center) {
            if let Some(_) = storage.get_chunk(center) {
                continue;
            }
            debug!("Spawning new chunk at center {:?}", center);

            let pos = storage.tile_to_world_pos(center).extend(0.0).xzy();
            let chunk_entity = commands
                .spawn((
                    ChunkCoord(center),
                    Transform::from_translation(pos),
                    Visibility::default(),
                    Name::new("TileChunk"),
                ))
                .id();
            storage.insert_chunk(center, chunk_entity);

            for tile in storage.chunk_tiles(center) {
                let pos = storage.tile_to_world_pos(tile - center);
                let local = tile - center;

                let tile_entity = commands
                    .spawn((
                        TileCoord(tile),
                        LocalTileCoord(local),
                        Transform::from_translation(pos.extend(0.0).xzy()),
                        Visibility::default(),
                        Name::new("Tile"),
                    ))
                    .id();
                commands.entity(chunk_entity).add_child(tile_entity);
            }

            for tile in storage.chunk_border(center) {
                let pos = storage.tile_to_world_pos(tile - center);
                let local = tile - center;

                let tile_entity = commands
                    .spawn((
                        TileBorder,
                        TileCoord(tile),
                        LocalTileCoord(local),
                        Transform::from_translation(pos.extend(0.0).xzy()),
                        Visibility::default(),
                        Name::new("TileBorder"),
                    ))
                    .id();
                commands.entity(chunk_entity).add_child(tile_entity);
            }
        }
    }
}

#[cfg(feature = "debug")]
mod debug {
    use super::{ChunkCoord, TileMapStorage};
    use bevy::prelude::*;

    #[derive(Debug, Resource, Default, Clone, Deref, DerefMut)]
    struct ShowGrid(pub bool);

    #[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
    pub struct DebugSet;

    pub struct DebugPlugin;

    impl Plugin for DebugPlugin {
        fn build(&self, app: &mut App) {
            app.insert_resource(ShowGrid(true));
            app.add_systems(Update, (toggle, draw_grid).in_set(DebugSet));
        }
    }

    fn toggle(kbd: Res<ButtonInput<KeyCode>>, mut show_grid: ResMut<ShowGrid>) {
        if kbd.just_pressed(KeyCode::F11) {
            show_grid.0 = !show_grid.0;
        }
    }

    fn draw_grid(
        mut gizmos: Gizmos,
        q_chunk: Query<&ChunkCoord>,
        show_grid: Res<ShowGrid>,
        storage: Res<TileMapStorage>,
    ) {
        if !**show_grid {
            return;
        }

        for chunk in q_chunk.iter() {
            for tile in storage.chunk_tiles(**chunk) {
                let pos = storage.tile_to_world_pos(tile).extend(0.0).xzy();
                draw_square(&mut gizmos, pos, storage.tile_size.x, Color::WHITE);
            }

            let pos = storage.tile_to_world_pos(**chunk).extend(0.0).xzy();
            draw_square(
                &mut gizmos,
                pos,
                storage.tile_size.x * 0.5,
                Color::srgb_u8(255, 255, 0),
            );
        }
    }

    fn draw_square(gizmos: &mut Gizmos, pos: Vec3, size: f32, color: Color) {
        let half_size = size / 2.0;
        let corners = [
            pos + Vec3::new(-half_size, 0.0, -half_size),
            pos + Vec3::new(half_size, 0.0, -half_size),
            pos + Vec3::new(half_size, 0.0, half_size),
            pos + Vec3::new(-half_size, 0.0, half_size),
        ];
        for i in 0..4 {
            let start = corners[i];
            let end = corners[(i + 1) % 4];
            gizmos.line(start, end, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_to_center() {
        let storage = TileMapStorage {
            tile_size: Vec2::splat(1.0),
            chunk_radius: 2,
            discover_radius: 1,
            chunks: HashMap::default(),
        };

        assert_eq!(storage.tile_to_center(&IVec2::new(0, 0)), IVec2::new(0, 0));
        assert_eq!(storage.tile_to_center(&IVec2::new(1, 1)), IVec2::new(0, 0));
        assert_eq!(storage.tile_to_center(&IVec2::new(2, 2)), IVec2::new(0, 0));
        assert_eq!(storage.tile_to_center(&IVec2::new(3, 3)), IVec2::new(5, 5));
        assert_eq!(storage.tile_to_center(&IVec2::new(-1, -1)), IVec2::new(0, 0));
        assert_eq!(storage.tile_to_center(&IVec2::new(-3, -3)), IVec2::new(-5, -5));
    }

    #[test]
    fn test_world_pos_to_tile() {
        let storage = TileMapStorage {
            tile_size: Vec2::splat(1.0),
            chunk_radius: 2,
            discover_radius: 1,
            chunks: HashMap::default(),
        };

        assert_eq!(storage.world_pos_to_tile(Vec2::new(0.0, 0.0)), IVec2::new(0, 0));
        assert_eq!(storage.world_pos_to_tile(Vec2::new(0.4, 0.4)), IVec2::new(0, 0));
        assert_eq!(storage.world_pos_to_tile(Vec2::new(0.5, 0.5)), IVec2::new(1, 1));
        assert_eq!(storage.world_pos_to_tile(Vec2::new(1.4, 1.4)), IVec2::new(1, 1));
        assert_eq!(storage.world_pos_to_tile(Vec2::new(-0.4, -0.4)), IVec2::new(0, 0));
        assert_eq!(storage.world_pos_to_tile(Vec2::new(-0.6, -0.6)), IVec2::new(-1, -1));
        assert_eq!(storage.world_pos_to_tile(Vec2::new(-1.4, -1.4)), IVec2::new(-1, -1));

        assert_eq!(storage.world_pos_to_tile(Vec2::new(-0.5, -0.5)), IVec2::new(0, 0));
    }

    #[test]
    fn test_tile_to_world_pos() {
        let storage = TileMapStorage {
            tile_size: Vec2::splat(1.0),
            chunk_radius: 2,
            discover_radius: 1,
            chunks: HashMap::default(),
        };

        assert_eq!(storage.tile_to_world_pos(IVec2::new(0, 0)), Vec2::new(0.0, 0.0));
        assert_eq!(storage.tile_to_world_pos(IVec2::new(1, 1)), Vec2::new(1.0, 1.0));
        assert_eq!(storage.tile_to_world_pos(IVec2::new(-1, -1)), Vec2::new(-1.0, -1.0));
        assert_eq!(storage.tile_to_world_pos(IVec2::new(5, 5)), Vec2::new(5.0, 5.0));
    }

    #[test]
    fn test_discover_chunks() {
        let storage = TileMapStorage {
            tile_size: Vec2::splat(1.0),
            chunk_radius: 2,
            discover_radius: 1,
            chunks: HashMap::default(),
        };

        let center = IVec2::new(0, 0);
        let chunks = storage.discover_chunks(center);
        let expected = vec![
            IVec2::new(-5, -5),
            IVec2::new(-5, 0),
            IVec2::new(-5, 5),
            IVec2::new(0, -5),
            IVec2::new(0, 0),
            IVec2::new(0, 5),
            IVec2::new(5, -5),
            IVec2::new(5, 0),
            IVec2::new(5, 5),
        ];
        assert_eq!(chunks, expected);
    }

    #[test]
    fn test_chunk_tiles() {
        let storage = TileMapStorage {
            tile_size: Vec2::splat(1.0),
            chunk_radius: 2,
            discover_radius: 1,
            chunks: HashMap::default(),
        };

        let center = IVec2::new(0, 0);
        let tiles = storage.chunk_tiles(center);
        let expected = vec![
            IVec2::new(-2, -2),
            IVec2::new(-2, -1),
            IVec2::new(-2, 0),
            IVec2::new(-2, 1),
            IVec2::new(-2, 2),
            IVec2::new(-1, -2),
            IVec2::new(-1, -1),
            IVec2::new(-1, 0),
            IVec2::new(-1, 1),
            IVec2::new(-1, 2),
            IVec2::new(0, -2),
            IVec2::new(0, -1),
            IVec2::new(0, 0),
            IVec2::new(0, 1),
            IVec2::new(0, 2),
            IVec2::new(1, -2),
            IVec2::new(1, -1),
            IVec2::new(1, 0),
            IVec2::new(1, 1),
            IVec2::new(1, 2),
            IVec2::new(2, -2),
            IVec2::new(2, -1),
            IVec2::new(2, 0),
            IVec2::new(2, 1),
            IVec2::new(2, 2),
        ];
        assert_eq!(tiles, expected);
    }

    fn test_chunk_border() {
        let storage = TileMapStorage {
            tile_size: Vec2::splat(1.0),
            chunk_radius: 2,
            discover_radius: 1,
            chunks: HashMap::default(),
        };

        let center = IVec2::new(0, 0);
        let border = storage.chunk_border(center);
        let expected = vec![
            IVec2::new(-3, -3),
            IVec2::new(-2, -3),
            IVec2::new(-1, -3),
            IVec2::new(0, -3),
            IVec2::new(1, -3),
            IVec2::new(2, -3),
            IVec2::new(3, -3),
            IVec2::new(-3, 3),
            IVec2::new(-2, 3),
            IVec2::new(-1, 3),
            IVec2::new(0, 3),
            IVec2::new(1, 3),
            IVec2::new(2, 3),
            IVec2::new(3, 3),
            IVec2::new(-3, -2),
            IVec2::new(-3, -1),
            IVec2::new(-3, 0),
            IVec2::new(-3, 1),
            IVec2::new(-3, 2),
            IVec2::new(3, -2),
            IVec2::new(3, -1),
            IVec2::new(3, 0),
            IVec2::new(3, 1),
            IVec2::new(3, 2),
        ];
        assert_eq!(border, expected);
    }
}
