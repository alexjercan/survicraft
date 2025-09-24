use bevy::prelude::*;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub(super) enum LauncherStates {
    #[default]
    Loading,
    MainMenu,
    Connecting,
    Generating,
    Playing,
}
