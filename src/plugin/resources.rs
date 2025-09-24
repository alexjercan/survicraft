use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Default)]
pub(super) enum LauncherMode {
    Client(String), // Server address
    #[default]
    Host,
    Dedicated,
}
