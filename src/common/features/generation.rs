use bevy::prelude::*;

use super::{components::*, planet::*, resources::*};
use crate::common::prelude::*;

pub struct FeaturesGenerationPlugin {}

impl Plugin for FeaturesGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChunkMapPlugin::<(TileCoord, Tile), TileFeature, _>::new(
            PlanetFeatures::default(),
        ));

        app.add_systems(
            Update,
            update_features_map.run_if(resource_changed::<FeatureAssets>),
        );
        app.add_systems(
            Update,
            update_terrain_seed.run_if(resource_changed::<TerrainGenerationSeed>),
        );
    }
}

fn update_features_map(features: Res<FeatureAssets>, mut planet_features: ResMut<PlanetFeatures>) {
    planet_features.map = features.clone();
}

fn update_terrain_seed(mut func: ResMut<PlanetFeatures>, seed: Res<TerrainGenerationSeed>) {
    *func = func.clone().with_seed(**seed + 1);
    debug!("Updated terrain seed to {}", seed.0 + 1);
}

impl ChunkMapInput for (TileCoord, Tile) {
    type Query = (&'static TileCoord, &'static Tile);

    fn from_query_item(
        item: bevy::ecs::query::QueryItem<<Self::Query as bevy::ecs::query::QueryData>::ReadOnly>,
    ) -> Self {
        (item.0.clone(), item.1.clone())
    }
}
