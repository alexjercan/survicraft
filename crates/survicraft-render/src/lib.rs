use bevy::prelude::*;
use survicraft_assets::{AssetState, AssetsPlugin, AssetsPluginSet};

pub mod main_menu;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum RenderState {
    #[default]
    Loading,
    MainMenu,
    InGame,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderPluginSet;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<RenderState>();

        app.add_plugins(AssetsPlugin);
        app.configure_sets(
            Update,
            AssetsPluginSet
                .run_if(in_state(RenderState::Loading))
                .in_set(RenderPluginSet),
        );
        app.add_systems(
            OnEnter(AssetState::Loaded),
            |mut state: ResMut<NextState<RenderState>>| {
                state.set(RenderState::MainMenu);
            },
        );

        app.add_plugins(main_menu::MainMenuPlugin);
        app.configure_sets(
            Update,
            main_menu::MainMenuPluginSet
                .run_if(in_state(RenderState::MainMenu))
                .in_set(RenderPluginSet),
        );
        app.add_systems(OnEnter(RenderState::MainMenu), menu_setup.in_set(RenderPluginSet));
    }
}

fn menu_setup(mut commands: Commands, mut menu_state: ResMut<NextState<main_menu::MenuState>>) {
    commands.spawn((
        Name::new("CameraUI"),
        Camera2d,
        StateScoped(RenderState::MainMenu),
    ));

    menu_state.set(main_menu::MenuState::Main);
}
