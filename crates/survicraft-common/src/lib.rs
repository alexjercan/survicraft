use bevy::prelude::*;

pub mod debug;
pub mod main_menu;
pub mod chat;
pub mod setup;

#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
pub enum DisplayQuality {
    Low,
    Medium,
    High,
}

#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy, Deref, DerefMut)]
pub struct Volume(pub u32);

#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Deref, DerefMut)]
pub struct PlayerName(pub String);
