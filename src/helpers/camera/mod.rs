//! TODO: Document this module

pub mod head_camera;
pub mod wasd_camera;

pub mod prelude {
    pub use super::head_camera::*;
    pub use super::wasd_camera::*;
}
