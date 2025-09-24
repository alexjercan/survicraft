use super::resources::*;
use bevy::prelude::*;

#[derive(Component, Debug, Clone, Deref, DerefMut, Reflect)]
pub struct TileFeature(pub Option<FeatureID>);
