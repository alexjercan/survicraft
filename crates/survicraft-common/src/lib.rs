use bevy::prelude::*;

pub mod debug;
pub mod main_menu;
pub mod setup;

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
