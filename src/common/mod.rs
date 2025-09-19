mod assets;
mod controller;
mod debug;
mod main_menu;
mod plugin;
mod server_plugin;
mod render;

pub mod prelude {
    pub use super::assets::*;
    pub use super::controller::*;
    pub use super::debug::*;
    pub use super::main_menu::*;
    pub use super::plugin::*;
    pub use super::server_plugin::*;
    pub use super::render::*;
}
