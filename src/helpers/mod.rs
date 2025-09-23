//! TODO: Add module-level documentation
//!
//! Simple helper utitlies that I can re-use in other games. Some important aspects:
//! - this should not depend on multiplayer stuff (e.g lightyear)

mod camera;
mod character;
mod chat;
mod chunk_map;
mod terrain;
mod tilemap;

mod glue;

pub mod prelude {
    pub use super::camera::prelude::*;
    pub use super::character::prelude::*;
    pub use super::chat::prelude::*;
    pub use super::chunk_map::prelude::*;
    pub use super::terrain::prelude::*;
    pub use super::tilemap::prelude::*;
}
