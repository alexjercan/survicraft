#![allow(dead_code)]

use bevy::prelude::*;

pub mod controller;
pub mod wasd;

/// Marker component for the player character entity. Spawn this when you
/// want to attach a player bundle and have it be controlled by a player.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerController;
