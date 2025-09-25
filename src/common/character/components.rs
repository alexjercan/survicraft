use bevy::prelude::*;

/// A marker component indicating that an entity is using a character controller.
#[derive(Component, Debug)]
pub struct CharacterController;

/// A component that holds the input state for a character controller.
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
pub struct CharacterInput {
    pub move_axis: Vec2,
    pub jump: bool,
}
