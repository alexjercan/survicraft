use bevy::prelude::*;
use super::resources::*;

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct Tile(pub TileID);

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut, Reflect)]
pub struct TileNoiseHeight(pub f64);

#[derive(Component, Debug, Clone, Copy)]
pub struct ChunkMesh;
