//! TODO: Document components module

use super::resources::*;
use bevy::prelude::*;

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct Tile(pub TileID);

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut, Reflect)]
pub struct TileNoiseHeight(pub f64);

#[derive(Component, Debug, Clone, Deref, DerefMut, Reflect)]
pub struct ChunkMesh(pub Mesh);

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct ChunkReady;

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut, Reflect)]
pub struct TileWorldHeight(pub f64);
