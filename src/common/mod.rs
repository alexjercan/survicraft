mod main_menu;
mod assets;
mod plugin;
mod player;

pub mod prelude {
    pub use super::main_menu::*;
    pub use super::assets::*;
    pub use super::plugin::*;
    pub use super::player::prelude::*;
}
