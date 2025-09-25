//! TODO: Document this module

mod components;
mod constants;
mod kinematic;
mod physics;

pub mod prelude {
    pub use super::components::*;
    pub use super::constants::{CHARACTER_CAPSULE_HEIGHT, CHARACTER_CAPSULE_RADIUS};
    pub use super::kinematic::prelude::*;
    pub use super::physics::prelude::*;
}
