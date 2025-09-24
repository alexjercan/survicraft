use super::{resources::*, states::*};
use crate::prelude::*;
use bevy::prelude::*;
use bevy_simple_text_input::TextInputPlugin;

pub(super) struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        // Add the TextInputPlugin to the Bevy app to enable text input functionality
        app.add_plugins(TextInputPlugin);

        // Setup the main menu UI for the MainMenu state
        app.add_systems(OnEnter(LauncherStates::MainMenu), setup_menu_ui);
        app.add_plugins(MainMenuPlugin);
        app.add_systems(
            Update,
            (handle_play_button_pressed, handle_multiplayer_pressed)
                .run_if(in_state(LauncherStates::MainMenu)),
        );

        // Setup the connecting UI for the Connecting state
        app.add_systems(OnEnter(LauncherStates::Connecting), setup_connecting_ui);

        // Setup the loading UI for the Loading state
        app.add_systems(OnEnter(LauncherStates::Generating), setup_loading_ui);

        // Chat setup. We set up chat UI and related systems.
        app.add_systems(OnEnter(LauncherStates::Playing), setup_chat);
        app.add_plugins(ChatPlugin);
    }
}

fn setup_menu_ui(mut commands: Commands, assets: Res<MainMenuAssets>) {
    debug!("Setting up main menu...");

    // Initialize the main menu resources from the loaded assets
    commands.insert_resource(MainMenuIcons {
        exit_icon: assets.exit_icon.clone(),
        right_icon: assets.right_icon.clone(),
        wrench_icon: assets.wrench_icon.clone(),
    });

    commands.spawn((
        Name::new("CameraMainMenuUI"),
        Camera2d,
        StateScoped(LauncherStates::MainMenu),
    ));

    // Spawn the main menu UI root node scoped to the MainMenu state.
    // This will trigger the MainMenuPlugin systems to populate it.
    commands.spawn((
        Name::new("MainMenuUI"),
        MainMenuRoot,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        StateScoped(LauncherStates::MainMenu),
    ));
}

fn handle_play_button_pressed(
    mut ev_play: EventReader<ClientPlayClickEvent>,
    mut next_state: ResMut<NextState<LauncherStates>>,
    mut mode: ResMut<LauncherMode>,
) {
    for _ in ev_play.read() {
        // If the play button is pressed, transition to the Playing state
        // We spawn the ServerListener and ClientConnection entities here
        // to enable hosting mode.

        next_state.set(LauncherStates::Connecting);
        *mode = LauncherMode::Host;
    }
}

fn handle_multiplayer_pressed(
    mut ev_multiplayer: EventReader<ClientMultiplayerClickEvent>,
    mut next_state: ResMut<NextState<LauncherStates>>,
    mut mode: ResMut<LauncherMode>,
) {
    for event in ev_multiplayer.read() {
        // If the multiplayer button is pressed, transition to the Playing state
        // We spawn only the ClientConnection entity here to enable joining mode.

        next_state.set(LauncherStates::Connecting);
        *mode = LauncherMode::Client(event.address.clone());
    }
}

fn setup_connecting_ui() {}

fn setup_loading_ui() {
    // TODO: Implement a proper loading UI
}

fn setup_chat(mut commands: Commands) {
    commands.spawn((
        Name::new("ChatUI"),
        ChatMenuRoot,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        StateScoped(LauncherStates::Playing),
    ));
}
