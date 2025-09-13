use bevy::prelude::*;

use crate::{chunk_map::ChunkMapInput, tilemap::tilemap::TileCoord};

pub mod main_menu;
pub mod setup;
pub mod chunk_map;
pub mod tilemap;
pub mod terrain;
pub mod camera;

// TODO: Might want to have something like a config module where I put the main menu and stuff that
// is not necessarily "helpers" but more like "core app stuff"
// TODO: I mean even terrain is kind of that in a way

#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
pub enum DisplayQualitySetting {
    Low,
    Medium,
    High,
}

#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy, Deref, DerefMut)]
pub struct VolumeSetting(pub u32);

#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Deref, DerefMut)]
pub struct PlayerNameSetting(pub String);

impl ChunkMapInput for TileCoord {
    type Query = (&'static TileCoord,);

    fn from_query_item(
        item: bevy::ecs::query::QueryItem<<Self::Query as bevy::ecs::query::QueryData>::ReadOnly>,
    ) -> Self {
        item.0.clone()
    }
}
