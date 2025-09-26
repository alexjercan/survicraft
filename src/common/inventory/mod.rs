mod components;
mod resources;
mod plugin;
mod render;

pub mod prelude {
    pub use super::components::{Item, Crafter, CrafterInput};
    pub use super::resources::*;
    pub use super::plugin::InventoryPlugin;
    pub use super::render::InventoryRenderPlugin;
}
