//! TODO: Add documentation

use super::{chunk_map::ChunkMapInput, tilemap::prelude::TileCoord};

impl ChunkMapInput for TileCoord {
    type Query = (&'static TileCoord,);

    fn from_query_item(
        item: bevy::ecs::query::QueryItem<<Self::Query as bevy::ecs::query::QueryData>::ReadOnly>,
    ) -> Self {
        item.0.clone()
    }
}
