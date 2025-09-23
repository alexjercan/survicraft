//! TODO: Add documentation

use super::{components::*, resources::*};
use crate::common::prelude::*;
use bevy::prelude::*;

pub struct FeaturesRenderPlugin {}

impl Default for FeaturesRenderPlugin {
    fn default() -> Self {
        Self {}
    }
}

impl Plugin for FeaturesRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_feature_tile);
    }
}

#[derive(Component)]
struct ChunkFeatureReady;

fn handle_feature_tile(
    mut commands: Commands,
    assets: Res<FeatureAssets>,
    q_hex: Query<(Entity, &TileWorldHeight, &Tile, &TileFeature), Without<ChunkFeatureReady>>,
) {
    if q_hex.is_empty() {
        return;
    }
    trace!("Handling feature tiles for {} hexes", q_hex.iter().len());

    for (entity, height, tile, feature) in q_hex.iter() {
        commands.entity(entity).insert(ChunkFeatureReady);

        let Some(id) = (**feature).clone() else {
            continue;
        };

        let Some(feature_asset) = assets.get_feature(&id) else {
            continue;
        };

        let Some(variant) = feature_asset.get_variant(&**tile) else {
            continue;
        };

        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Transform::from_xyz(0.0, (**height) as f32, 0.0).with_scale(Vec3::splat(3.0)),
                SceneRoot(variant.scene.clone()),
                Name::new("Feature Tile"),
            ));
        });
    }
}
