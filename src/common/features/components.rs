use bevy::prelude::*;

use super::resources::*;

#[derive(Component, Debug, Clone, Deref, DerefMut, Reflect)]
pub struct TileFeature(pub Option<FeatureID>);
