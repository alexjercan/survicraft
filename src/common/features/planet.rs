use bevy::prelude::*;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin, Worley};

use super::{components::*, resources::*};
use crate::common::prelude::*;

/// Planet seed. Change this to generate a different planet.
const CURRENT_SEED: u32 = 0;

/// Scale of the planet. Change this to zoom in or out.
const ZOOM_SCALE: f64 = 0.001;

/// Frequency of the planet's feature patches. This value is measured in
/// radians. Higher frequency produces smaller, more numerous patches.
const FEATURE_PATCH_FREQUENCY: f64 = 0.125;

/// Lacunarity of the planet's feature patches. Changing this value
/// produces slightly different patches. For the best results, this value
/// should be random, but close to 2.0.
const FEATURE_PATCH_LACUNARITY: f64 = 2.1875;

/// Frequency of the planet's features abundance. This value is measured
/// in radians. Higher frequency produces smaller, more numerous abundance.
const FEATURE_ABUNDANCE_FREQUENCY: f64 = 0.4;

/// Lacunarity of the planet's features abundance. Changing this value
/// produces slightly different abundance. For the best results, this
/// value should be random, but close to 2.0.
const FEATURE_ABUNDANCE_LACUNARITY: f64 = 2.21875;

#[derive(Resource, Debug, Clone)]
pub struct PlanetFeatures {
    seed: u32,
    zoom_scale: f64,
    pub map: FeatureAssets,
    patch_frequency: f64,
    patch_lacunarity: f64,
    abundance_frequency: f64,
    abundance_lacunarity: f64,
}

impl Default for PlanetFeatures {
    fn default() -> Self {
        PlanetFeatures {
            seed: CURRENT_SEED,
            zoom_scale: ZOOM_SCALE,
            map: FeatureAssets::default(),
            patch_frequency: FEATURE_PATCH_FREQUENCY,
            patch_lacunarity: FEATURE_PATCH_LACUNARITY,
            abundance_frequency: FEATURE_ABUNDANCE_FREQUENCY,
            abundance_lacunarity: FEATURE_ABUNDANCE_LACUNARITY,
        }
    }
}

impl PlanetFeatures {
    pub fn with_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }
}

impl ChunkMapFunction<(TileCoord, Tile), TileFeature> for PlanetFeatures {
    fn get(&self, (point, tile): (TileCoord, Tile)) -> TileFeature {
        let x = point.x as f64 * self.zoom_scale;
        let y = point.y as f64 * self.zoom_scale;

        // 1. Voronoi-like patch selection using Worley noise
        let selection_noise = Fbm::<Worley>::new(self.seed)
            .set_frequency(self.patch_frequency)
            .set_persistence(0.5)
            .set_lacunarity(self.patch_lacunarity)
            .set_octaves(8)
            .get([x, y]);

        let feature_index =
            ((selection_noise + 1.0) * 0.5 * self.map.features.len() as f64) as usize;
        let feature_index = feature_index.min(self.map.features.len() - 1);

        // 2. Abundance variation using high-frequency FBM
        let abundance_val = Fbm::<Perlin>::new(self.seed + 1)
            .set_frequency(self.abundance_frequency)
            .set_persistence(0.5)
            .set_lacunarity(self.abundance_lacunarity)
            .set_octaves(4)
            .get([x, y]);
        let abundance_val = (abundance_val + 1.0) * 0.5;

        let feature = self.map.features[feature_index].clone();
        let variant = feature.get_variant(&tile);

        if let Some(variant) = variant {
            if variant.threshold <= abundance_val {
                return TileFeature(Some(feature.id));
            }
        }

        TileFeature(None)
    }
}
