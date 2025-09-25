#![allow(clippy::type_complexity, clippy::too_many_arguments)]

mod common;
mod plugin;

pub mod prelude {
    pub use super::common::prelude::*;
    pub use super::plugin::{new_gui_app, new_headless_app, LauncherPlugin};
}
