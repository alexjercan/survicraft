use bevy::prelude::*;

use crate::common::prelude::*;

pub type FeatureID = String;

#[derive(Asset, TypePath, Debug, Clone)]
pub struct FeatureAsset {
    pub id: FeatureID,
    pub name: String,
    pub variants: Vec<FeatureVariant>,
}

impl FeatureAsset {
    pub fn get_variant(&self, id: &TileID) -> Option<&FeatureVariant> {
        self.variants.iter().find(|variant| &variant.id == id)
    }
}

#[derive(Debug, Clone)]
pub struct FeatureVariant {
    pub id: TileID,
    pub name: String,
    pub threshold: f64,
    pub scene: Handle<Scene>,
}

#[derive(Resource, Clone, Default, Debug)]
pub struct FeatureAssets {
    pub features: Vec<FeatureAsset>,
}

impl FeatureAssets {
    pub fn new(features: Vec<FeatureAsset>) -> Self {
        Self { features }
    }

    pub fn get_feature(&self, id: &FeatureID) -> Option<&FeatureAsset> {
        self.features.iter().find(|feature| &feature.id == id)
    }
}
