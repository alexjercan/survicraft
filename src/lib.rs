mod common;
mod plugin;

pub mod prelude {
    pub use super::common::prelude::*;
    pub use super::plugin::LauncherPlugin;
}
