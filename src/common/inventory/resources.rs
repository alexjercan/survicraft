use avian3d::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};

pub type ItemID = String;

#[derive(Asset, TypePath, Debug, Clone)]
pub struct ItemAsset {
    pub id: ItemID,
    pub name: String,
    pub description: String,
    pub weight: f32,
    pub scene: Handle<Scene>,
    pub offset: Option<Vec3>,
    pub scale: Option<Vec3>,
    pub collider: Option<Collider>,
    pub modifiers: Vec<ItemModifier>,
}

#[derive(Debug, Clone)]
pub enum ItemModifier {
    Storage { capacity: u32 },
}

#[derive(Resource, Clone, Default, Debug)]
pub struct ItemAssets {
    pub items: Vec<ItemAsset>,
}

impl ItemAssets {
    pub fn new(items: Vec<ItemAsset>) -> Self {
        Self { items }
    }

    pub fn get_item(&self, id: &ItemID) -> Option<&ItemAsset> {
        self.items.iter().find(|item| &item.id == id)
    }
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct RecipeAsset {
    pub input: Vec<RecipeItem>,
    pub output: Vec<RecipeItem>,
}

impl RecipeAsset {
    pub fn can_craft(&self, available: &HashMap<ItemID, u32>) -> bool {
        for item in &self.input {
            let count = available.get(&item.item_id).cloned().unwrap_or(0);
            if count < item.count {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone)]
pub struct RecipeItem {
    pub item_id: ItemID,
    pub count: u32,
}

#[derive(Resource, Clone, Default, Debug)]
pub struct RecipeAssets {
    pub recipes: Vec<RecipeAsset>,
}

impl RecipeAssets {
    pub fn new(recipes: Vec<RecipeAsset>) -> Self {
        Self { recipes }
    }
}
