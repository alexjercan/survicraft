use bevy::{prelude::*, render::view::RenderLayers};
use bevy_simple_text_input::TextInputPlugin;

use super::{progress::*, resources::*, states::*};
use crate::prelude::*;

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
        app.add_systems(OnEnter(LauncherStates::Generating), setup_generating_ui);
        app.add_systems(
            Update,
            update_generating_ui.run_if(in_state(LauncherStates::Generating)),
        );

        // Chat setup. We set up chat UI and related systems.
        app.add_systems(OnEnter(LauncherStates::Playing), setup_playing_ui);
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

fn setup_connecting_ui(mut commands: Commands) {
    commands.spawn((
        Name::new("CameraConnectingUI"),
        Camera2d,
        StateScoped(LauncherStates::Connecting),
    ));

    commands.spawn((
        Name::new("ConnectingUIRoot"),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        StateScoped(LauncherStates::Connecting),
    ));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GeneratingUIProgressBar;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GeneratingUIProgressFill;

fn setup_generating_ui(mut commands: Commands) {
    commands.spawn((
        Name::new("CameraLoadingUI"),
        Camera2d,
        StateScoped(LauncherStates::Generating),
    ));

    commands
        .spawn((
            Name::new("GeneratingUIRoot"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            StateScoped(LauncherStates::Generating),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("GeneratingProgressBar"),
                    GeneratingUIProgressBar,
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(30.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Name::new("GeneratingProgressFill"),
                        GeneratingUIProgressFill,
                        Node {
                            width: Val::Percent(0.0), // Start with 0% width
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.7, 0.2)),
                    ));
                });
        });
}

fn update_generating_ui(
    terrain_progress: Res<ProgressGeneration>,
    mut q_fill: Query<&mut Node, With<GeneratingUIProgressFill>>,
) {
    if terrain_progress.is_changed() {
        for mut node in &mut q_fill {
            node.width = Val::Percent(terrain_progress.clamp(0.0, 1.0) * 100.0);
        }
    }
}

fn setup_playing_ui(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
        Name::new("UI Camera"),
        RenderLayers::layer(1),
        StateScoped(LauncherStates::Playing),
    ));

    commands
        .spawn((
            Name::new("ChatUIRoot"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            StateScoped(LauncherStates::Playing),
        ))
        .with_children(|parent| {
            // --- Chat history in bottom-left ---
            parent.spawn((
                Name::new("ChatHistoryUIRoot"),
                ChatHistoryRoot,
                Node {
                    width: Val::Px(300.0),
                    height: Val::Px(200.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(20.0),
                    bottom: Val::Px(20.0),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow {
                        x: OverflowAxis::Hidden,
                        y: OverflowAxis::Scroll,
                    },
                    ..default()
                },
            ));

            // --- Chat input in middle-screen ---
            parent.spawn((
                Name::new("ChatInputUIRoot"),
                ChatInputRoot,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Relative,
                    ..default()
                },
            ));
        });

    // --- Status bar in top-right for FPS, latency, etc ---
    commands.spawn((
        Name::new("StatusBarUIRoot"),
        StatusBarRoot,
        Node {
            width: Val::Auto,
            height: Val::Auto,
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexEnd,
            ..default()
        },
    ));
}
