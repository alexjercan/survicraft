mod main_menu;
mod assets;
mod plugin;
mod server_plugin;
mod controller;
mod debug;

pub mod prelude {
    pub use super::main_menu::*;
    pub use super::assets::*;
    pub use super::plugin::*;
    pub use super::server_plugin::*;
    pub use super::controller::*;
    pub use super::debug::*;
}
