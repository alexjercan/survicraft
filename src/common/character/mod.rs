//! TODO: Document this module

mod physics;
mod kinematic;
mod constants;
mod components;

pub mod prelude {
    pub use super::physics::prelude::*;
    pub use super::kinematic::prelude::*;
    pub use super::constants::{CHARACTER_CAPSULE_HEIGHT, CHARACTER_CAPSULE_RADIUS};
    pub use super::components::*;
}
