mod main_menu;
mod assets;
mod plugin;
mod server_plugin;
mod player;
mod controller;

pub mod prelude {
    pub use super::main_menu::*;
    pub use super::assets::*;
    pub use super::plugin::*;
    pub use super::server_plugin::*;
    pub use super::player::prelude::*;
    pub use super::controller::*;
}
