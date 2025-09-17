//! TODO: Add module-level documentation

mod setup;
mod tilemap;
mod camera;
mod terrain;
mod chunk_map;
mod chat;
mod player;

mod glue;

pub mod prelude {
    pub use super::setup::*;
    pub use super::tilemap::prelude::*;
    pub use super::camera::prelude::*;
    pub use super::terrain::prelude::*;
    pub use super::chunk_map::prelude::*;
    pub use super::chat::prelude::*;
    pub use super::player::prelude::*;
}
