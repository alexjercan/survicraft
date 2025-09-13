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

use bevy::{platform::collections::HashMap, prelude::*};
// use pathfinding::prelude::astar;

#[cfg(feature = "debug")]
use self::debug::{DebugPlugin, DebugSet};

/// The TileDiscoverEvent is used to trigger the discovery of square tiles in the map.
/// The position is given in world coordinates, and the event is generic over a component type `C`
/// that can be constructed from a `IVec2` coordinate.
#[derive(Event, Clone, Debug)]
pub struct TileDiscoverEvent<C: From<IVec2>> {
    /// The position in world coordinates where the discovery event occurs.
    pub pos: Vec2,
    _marker: std::marker::PhantomData<C>,
}

impl<C: From<IVec2>> TileDiscoverEvent<C> {
    /// Creates a new TileDiscoverEvent with the specified position.
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            _marker: std::marker::PhantomData,
        }
    }
}

#[derive(Component, Clone, Debug, Deref, DerefMut)]
struct ChunkCoord(pub IVec2);

#[derive(Resource, Debug, Clone)]
pub struct TileMapStorage {
    tile_size: Vec2,
    chunk_radius: u32,
    discover_radius: u32,
    chunks: HashMap<IVec2, Entity>,
    tiles: HashMap<IVec2, Entity>,
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
        for x in -((self.chunk_radius as i32))..=(self.chunk_radius as i32) {
            for y in -((self.chunk_radius as i32))..=(self.chunk_radius as i32) {
                let tile_x = center.x + x;
                let tile_y = center.y + y;
                tiles.push(IVec2::new(tile_x, tile_y));
            }
        }
        tiles
    }

    fn tile_to_center(&self, tile: &IVec2) -> IVec2 {
        let step = self.chunk_radius as i32 * 2 + 1;
        IVec2::new(
            (tile.x / step) * step,
            (tile.y / step) * step,
        )
    }

    pub fn world_pos_to_tile(&self, position: Vec2) -> IVec2 {
        IVec2::new(
            (position.x / self.tile_size.x).floor() as i32,
            (position.y / self.tile_size.y).floor() as i32,
        )
    }

    pub fn tile_to_world_pos(&self, tile: IVec2) -> Vec2 {
        Vec2::new(
            tile.x as f32 * self.tile_size.x + self.tile_size.x / 2.0,
            tile.y as f32 * self.tile_size.y + self.tile_size.y / 2.0,
        )
    }

    fn get_chunk(&self, tile: IVec2) -> Option<&Entity> {
        self.chunks.get(&tile)
    }

    fn insert_chunk(&mut self, center: IVec2, chunk: Entity) {
        self.chunks.insert(center, chunk);
    }

    fn insert_tile(&mut self, tile: IVec2, entity: Entity) {
        self.tiles.insert(tile, entity);
    }

    pub fn get_tile(&self, tile: IVec2) -> Option<&Entity> {
        self.tiles.get(&tile)
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
pub struct TileMapPlugin<C: From<IVec2>> {
    tile_size: Vec2,
    chunk_radius: u32,
    discover_radius: u32,
    _marker: std::marker::PhantomData<C>,
}

impl<C: From<IVec2>> TileMapPlugin<C> {
    pub fn new(tile_size: Vec2, chunk_radius: u32, discover_radius: u32) -> Self {
        Self {
            tile_size,
            chunk_radius,
            discover_radius,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<C: Component + From<IVec2> + Send + Sync + 'static> Plugin for TileMapPlugin<C> {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        app.add_plugins(DebugPlugin);
        #[cfg(feature = "debug")]
        app.configure_sets(Update, DebugSet.in_set(TileMapSet));

        app.add_event::<TileDiscoverEvent<C>>();

        app.insert_resource(TileMapStorage {
            tile_size: self.tile_size,
            chunk_radius: self.chunk_radius,
            discover_radius: self.discover_radius,
            chunks: HashMap::default(),
            tiles: HashMap::default(),
        });

        app.add_systems(Update, (generate_chunks::<C>).in_set(TileMapSet).chain());
    }
}

fn generate_chunks<C: Component + From<IVec2> + Send + Sync + 'static>(
    mut commands: Commands,
    mut storage: ResMut<TileMapStorage>,
    mut ev_discover: EventReader<TileDiscoverEvent<C>>,
) {
    for ev in ev_discover.read() {
        let tile = storage.world_pos_to_tile(ev.pos);
        let center = storage.tile_to_center(&tile);

        for center in storage.discover_chunks(center) {
            if let Some(_) = storage.get_chunk(center) {
                continue;
            }

            let pos = storage.tile_to_world_pos(center).extend(0.0).xzy();
            let chunk_entity = commands
                .spawn((
                    ChunkCoord(center),
                    Transform::from_translation(pos),
                    Visibility::default(),
                    Name::new("HexChunk"),
                ))
                .id();
            storage.insert_chunk(center, chunk_entity);

            for tile in storage.chunk_tiles(center) {
                let pos = storage.tile_to_world_pos(tile - center).extend(0.0).xzy();

                let tile_entity = commands
                    .spawn((
                        C::from(tile),
                        Transform::from_translation(pos),
                        Visibility::default(),
                        Name::new("Hex"),
                    ))
                    .id();
                commands.entity(chunk_entity).add_child(tile_entity);
                storage.insert_tile(tile, tile_entity);
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
        if kbd.just_pressed(KeyCode::F12) {
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
