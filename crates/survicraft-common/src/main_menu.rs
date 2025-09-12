use bevy::{prelude::*, ui::FocusPolicy};
use bevy_simple_text_input::*;
use std::fmt::Debug;

use crate::{DisplayQuality, PlayerName, Volume};

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

const BORDER_COLOR_INACTIVE: Color = Color::srgb(0.25, 0.25, 0.25);
const BORDER_COLOR_ACTIVE: Color = Color::srgb(0.75, 0.52, 0.99);

const BACKGROUND_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

/// Marker component for the root UI node
/// Add this component to an entity to make it the root of the UI and spawn the main menu
#[derive(Debug, Clone, Component)]
pub struct MainMenuRoot;

#[derive(Resource, Default)]
pub struct MainMenuAssets {
    pub exit_icon: Handle<Image>,
    pub right_icon: Handle<Image>,
    pub wrench_icon: Handle<Image>,
}

/// Event that is triggered when the "Play" button is clicked in the main menu
#[derive(Debug, Clone, Event)]
pub struct ClientPlayClickEvent;

/// Event that is triggered when the "Connect" button is clicked in the multiplayer menu
#[derive(Debug, Clone, Event)]
pub struct ClientMultiplayerClickEvent {
    pub address: String,
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
    #[default]
    Main,
    Multiplayer,
    Settings,
    SettingsDisplay,
    SettingsSound,
    SettingsName,
}

#[derive(Component, Clone, Copy, Debug)]
struct MenuItem(MenuState);

#[derive(Component, Clone, Copy, Debug)]
struct NameInput;

#[derive(Component, Clone, Copy, Debug)]
struct AddressInput;

#[derive(Component)]
struct SelectedOption;

#[derive(Component)]
enum MenuButtonAction {
    Play,
    Multiplayer,
    MultiplayerConnect,
    Settings,
    SettingsDisplay,
    SettingsSound,
    SettingsName,
    BackToMainMenu,
    BackToSettings,
    Quit,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MainMenuPluginSet;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ClientPlayClickEvent>();
        app.add_event::<ClientMultiplayerClickEvent>();

        app.init_state::<MenuState>();

        app.insert_resource(DisplayQuality::Medium);
        app.insert_resource(Volume(7));
        app.insert_resource(PlayerName("Player".to_string()));

        app.add_systems(
            Update,
            (
                main_menu_setup,
                multiplayer_menu_setup,
                settings_menu_setup,
                display_settings_menu_setup,
                sound_settings_menu_setup,
                name_settings_menu_setup,
                handle_visible_menus.run_if(state_changed::<MenuState>),
                menu_action,
                handle_button_interact,
                handle_text_interact.before(TextInputSystem),
            )
                .in_set(MainMenuPluginSet),
        );
        app.add_systems(
            Update,
            (
                setting_button::<DisplayQuality>
                    .run_if(in_state(MenuState::SettingsDisplay))
                    .in_set(MainMenuPluginSet),
                setting_button::<Volume>
                    .run_if(in_state(MenuState::SettingsSound))
                    .in_set(MainMenuPluginSet),
                name_settings_menu_update
                    .run_if(in_state(MenuState::SettingsName))
                    .in_set(MainMenuPluginSet),
            ),
        );
    }
}

fn handle_button_interact(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color, selected) in &mut interaction_query {
        debug!("Button interaction: {interaction:?}");
        *background_color = match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

fn handle_text_interact(
    query: Query<(Entity, &Interaction), Changed<Interaction>>,
    mut text_input_query: Query<(Entity, &mut TextInputInactive, &mut BorderColor)>,
) {
    for (interaction_entity, interaction) in &query {
        debug!("Text interaction: {interaction:?}");
        if *interaction == Interaction::Pressed {
            for (entity, mut inactive, mut border_color) in &mut text_input_query {
                if entity == interaction_entity {
                    inactive.0 = false;
                    *border_color = BORDER_COLOR_ACTIVE.into();
                } else {
                    inactive.0 = true;
                    *border_color = BORDER_COLOR_INACTIVE.into();
                }
            }
        }
    }
}

fn setting_button<T: Resource + Component + PartialEq + Clone + Debug>(
    interaction_query: Query<(&Interaction, &T, Entity), (Changed<Interaction>, With<Button>)>,
    selected_query: Single<(Entity, &mut BackgroundColor), (With<T>, With<SelectedOption>)>,
    mut commands: Commands,
    mut setting: ResMut<T>,
) {
    let (previous_button, mut previous_button_color) = selected_query.into_inner();
    for (interaction, button_setting, entity) in &interaction_query {
        if *interaction == Interaction::Pressed && *setting != *button_setting {
            *previous_button_color = NORMAL_BUTTON.into();
            commands.entity(previous_button).remove::<SelectedOption>();
            commands.entity(entity).insert(SelectedOption);
            *setting = button_setting.clone();
            debug!("Setting changed to {:?}", setting);
        }
    }
}

fn name_settings_menu_update(
    query: Query<&TextInputValue, (Changed<TextInputValue>, With<NameInput>)>,
    mut player_name: ResMut<PlayerName>,
) {
    for text_input_value in query {
        player_name.0 = text_input_value.0.clone();
        debug!("Player name changed to {:?}", player_name.0);
    }
}

fn main_menu_setup(
    mut commands: Commands,
    assets: Res<MainMenuAssets>,
    root: Single<Entity, (With<MainMenuRoot>, Added<MainMenuRoot>)>,
) {
    // Common style for all buttons on the screen
    let button_node = Node {
        width: Val::Px(300.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_icon_node = Node {
        width: Val::Px(30.0),
        // This takes the icons out of the flexbox flow, to be positioned exactly
        position_type: PositionType::Absolute,
        // The icon will be close to the left border of the button
        left: Val::Px(10.0),
        ..default()
    };
    let button_text_font = TextFont {
        font_size: 33.0,
        ..default()
    };

    commands.entity(root.entity()).with_children(|parent| {
        parent
            .spawn((
                Name::new("MainMenu"),
                MenuItem(MenuState::Main),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent
                    .spawn((Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },))
                    .with_children(|parent| {
                        // Display the game name
                        parent.spawn((
                            Text::new("Survicraft"),
                            TextFont {
                                font_size: 67.0,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                            Node {
                                margin: UiRect::all(Val::Px(50.0)),
                                ..default()
                            },
                        ));

                        // Display three buttons for each action available from the main menu:
                        // - new game
                        // - multiplayer
                        // - settings
                        // - quit
                        parent
                            .spawn((
                                Name::new("PlayButton"),
                                Button,
                                button_node.clone(),
                                BackgroundColor(NORMAL_BUTTON),
                                MenuButtonAction::Play,
                            ))
                            .with_children(|parent| {
                                let icon = assets.right_icon.clone();
                                parent.spawn((ImageNode::new(icon), button_icon_node.clone()));
                                parent.spawn((
                                    Text::new("New Game"),
                                    button_text_font.clone(),
                                    TextColor(TEXT_COLOR),
                                ));
                            });

                        parent
                            .spawn((
                                Name::new("MultiplayerButton"),
                                Button,
                                button_node.clone(),
                                BackgroundColor(NORMAL_BUTTON),
                                MenuButtonAction::Multiplayer,
                            ))
                            .with_children(|parent| {
                                let icon = assets.right_icon.clone();
                                parent.spawn((ImageNode::new(icon), button_icon_node.clone()));
                                parent.spawn((
                                    Text::new("Multiplayer"),
                                    button_text_font.clone(),
                                    TextColor(TEXT_COLOR),
                                ));
                            });

                        parent
                            .spawn((
                                Name::new("SettingsButton"),
                                Button,
                                button_node.clone(),
                                BackgroundColor(NORMAL_BUTTON),
                                MenuButtonAction::Settings,
                            ))
                            .with_children(|parent| {
                                let icon = assets.wrench_icon.clone();
                                parent.spawn((ImageNode::new(icon), button_icon_node.clone()));
                                parent.spawn((
                                    Text::new("Settings"),
                                    button_text_font.clone(),
                                    TextColor(TEXT_COLOR),
                                ));
                            });

                        parent
                            .spawn((
                                Name::new("QuitButton"),
                                Button,
                                button_node,
                                BackgroundColor(NORMAL_BUTTON),
                                MenuButtonAction::Quit,
                            ))
                            .with_children(|parent| {
                                let icon = assets.exit_icon.clone();
                                parent.spawn((ImageNode::new(icon), button_icon_node));
                                parent.spawn((
                                    Text::new("Quit"),
                                    button_text_font,
                                    TextColor(TEXT_COLOR),
                                ));
                            });
                    });
            });
    });
}

fn multiplayer_menu_setup(
    mut commands: Commands,
    root: Single<Entity, (With<MainMenuRoot>, Added<MainMenuRoot>)>,
) {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = (
        TextFont {
            font_size: 33.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
    );

    commands.entity(root.entity()).with_children(|parent| {
        parent
            .spawn((
                Name::new("MultiplayerMenu"),
                MenuItem(MenuState::Multiplayer),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    display: Display::None,
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent
                    .spawn((Text::new("Multiplayer"), button_text_style.clone()))
                    .insert(Node {
                        margin: UiRect::all(Val::Px(50.0)),
                        ..default()
                    });

                parent.spawn((
                    Name::new("AddressInput"),
                    AddressInput,
                    Node {
                        width: Val::Px(500.0),
                        border: UiRect::all(Val::Px(5.0)),
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    Interaction::None,
                    BorderColor(BORDER_COLOR_INACTIVE),
                    BackgroundColor(BACKGROUND_COLOR),
                    FocusPolicy::Block,
                    TextInput,
                    TextInputTextFont(TextFont {
                        font_size: 34.,
                        ..default()
                    }),
                    TextInputTextColor(TextColor(TEXT_COLOR)),
                    TextInputValue("127.0.0.1".to_string()),
                    TextInputSettings {
                        retain_on_submit: true,
                        ..default()
                    },
                    TextInputInactive(true),
                ));

                parent
                    .spawn((Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },))
                    .with_children(|parent| {
                        for (action, text) in [
                            (MenuButtonAction::MultiplayerConnect, "Connect"),
                            (MenuButtonAction::BackToMainMenu, "Back"),
                        ] {
                            parent
                                .spawn((
                                    Button,
                                    button_node.clone(),
                                    BackgroundColor(NORMAL_BUTTON),
                                    action,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((Text::new(text), button_text_style.clone()));
                                });
                        }
                    });
            });
    });
}

fn settings_menu_setup(
    mut commands: Commands,
    root: Single<Entity, (With<MainMenuRoot>, Added<MainMenuRoot>)>,
) {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = (
        TextFont {
            font_size: 33.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
    );

    commands.entity(root.entity()).with_children(|parent| {
        parent
            .spawn((
                Name::new("SettingsMenu"),
                MenuItem(MenuState::Settings),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    display: Display::None,
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent
                    .spawn((Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },))
                    .with_children(|parent| {
                        for (action, text) in [
                            (MenuButtonAction::SettingsDisplay, "Display"),
                            (MenuButtonAction::SettingsSound, "Sound"),
                            (MenuButtonAction::SettingsName, "Name"),
                            (MenuButtonAction::BackToMainMenu, "Back"),
                        ] {
                            parent
                                .spawn((
                                    Button,
                                    button_node.clone(),
                                    BackgroundColor(NORMAL_BUTTON),
                                    action,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((Text::new(text), button_text_style.clone()));
                                });
                        }
                    });
            });
    });
}

fn display_settings_menu_setup(
    mut commands: Commands,
    display_quality: Res<DisplayQuality>,
    root: Single<Entity, (With<MainMenuRoot>, Added<MainMenuRoot>)>,
) {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = (
        TextFont {
            font_size: 33.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
    );

    commands.entity(root.entity()).with_children(|parent| {
        parent
            .spawn((
                Name::new("DisplaySettingsMenu"),
                MenuItem(MenuState::SettingsDisplay),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    display: Display::None,
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent
                    .spawn((Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },))
                    .with_children(|parent| {
                        // Create a new `Node`, this time not setting its `flex_direction`. It will
                        // use the default value, `FlexDirection::Row`, from left to right.
                        parent
                            .spawn((Node {
                                align_items: AlignItems::Center,
                                ..default()
                            },))
                            .with_children(|parent| {
                                // Display a label for the current setting
                                parent.spawn((
                                    Text::new("Display Quality"),
                                    button_text_style.clone(),
                                ));
                                // Display a button for each possible value
                                for quality_setting in [
                                    DisplayQuality::Low,
                                    DisplayQuality::Medium,
                                    DisplayQuality::High,
                                ] {
                                    let mut entity = parent.spawn((
                                        Button,
                                        Node {
                                            width: Val::Px(150.0),
                                            height: Val::Px(65.0),
                                            ..button_node.clone()
                                        },
                                        BackgroundColor(NORMAL_BUTTON),
                                        quality_setting,
                                    ));
                                    entity.with_children(|parent| {
                                        parent.spawn((
                                            Text::new(format!("{quality_setting:?}")),
                                            button_text_style.clone(),
                                        ));
                                    });
                                    if *display_quality == quality_setting {
                                        entity.insert(SelectedOption);
                                    }
                                }
                            });
                        // Display the back button to return to the settings screen
                        parent
                            .spawn((
                                Button,
                                button_node,
                                BackgroundColor(NORMAL_BUTTON),
                                MenuButtonAction::BackToSettings,
                            ))
                            .with_children(|parent| {
                                parent.spawn((Text::new("Back"), button_text_style));
                            });
                    });
            });
    });
}

fn sound_settings_menu_setup(
    mut commands: Commands,
    volume: Res<Volume>,
    root: Single<Entity, (With<MainMenuRoot>, Added<MainMenuRoot>)>,
) {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = (
        TextFont {
            font_size: 33.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
    );

    commands.entity(root.entity()).with_children(|parent| {
        parent
            .spawn((
                Name::new("SoundSettingsMenu"),
                MenuItem(MenuState::SettingsSound),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    display: Display::None,
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent
                    .spawn((Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },))
                    .with_children(|parent| {
                        parent
                            .spawn((Node {
                                align_items: AlignItems::Center,
                                ..default()
                            },))
                            .with_children(|parent| {
                                parent.spawn((Text::new("Volume"), button_text_style.clone()));
                                for volume_setting in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] {
                                    let mut entity = parent.spawn((
                                        Button,
                                        Node {
                                            width: Val::Px(30.0),
                                            height: Val::Px(65.0),
                                            ..button_node.clone()
                                        },
                                        BackgroundColor(NORMAL_BUTTON),
                                        Volume(volume_setting),
                                    ));
                                    if *volume == Volume(volume_setting) {
                                        entity.insert(SelectedOption);
                                    }
                                }
                            });
                        parent
                            .spawn((
                                Button,
                                button_node,
                                BackgroundColor(NORMAL_BUTTON),
                                MenuButtonAction::BackToSettings,
                            ))
                            .with_child((Text::new("Back"), button_text_style));
                    });
            });
    });
}

fn name_settings_menu_setup(
    mut commands: Commands,
    player_name: Res<PlayerName>,
    root: Single<Entity, (With<MainMenuRoot>, Added<MainMenuRoot>)>,
) {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = (
        TextFont {
            font_size: 33.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
    );

    commands.entity(root.entity()).with_children(|parent| {
        parent
            .spawn((
                Name::new("NameSettingsMenu"),
                MenuItem(MenuState::SettingsName),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    display: Display::None,
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent
                    .spawn((Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },))
                    .with_children(|parent| {
                        parent.spawn((Text::new("Player Name"), button_text_style.clone()));
                        parent.spawn((
                            NameInput,
                            Node {
                                width: Val::Px(300.0),
                                border: UiRect::all(Val::Px(5.0)),
                                padding: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            Interaction::None,
                            BorderColor(BORDER_COLOR_INACTIVE),
                            BackgroundColor(BACKGROUND_COLOR),
                            FocusPolicy::Block,
                            TextInput,
                            TextInputTextFont(TextFont {
                                font_size: 34.,
                                ..default()
                            }),
                            TextInputTextColor(TextColor(TEXT_COLOR)),
                            TextInputValue(player_name.0.clone()),
                            TextInputSettings {
                                retain_on_submit: true,
                                ..default()
                            },
                        ));
                        parent
                            .spawn((
                                Button,
                                button_node,
                                BackgroundColor(NORMAL_BUTTON),
                                MenuButtonAction::BackToSettings,
                            ))
                            .with_child((Text::new("Back"), button_text_style));
                    });
            });
    });
}

fn handle_visible_menus(
    menu_state: Res<State<MenuState>>,
    mut query: Query<(&MenuItem, &mut Node)>,
) {
    for (menu_item, mut node) in &mut query {
        if menu_item.0 == **menu_state {
            node.display = Display::Flex;
        } else {
            node.display = Display::None;
        }
    }
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    q_address: Query<&TextInputValue, With<AddressInput>>,
    mut play_ev: EventWriter<ClientPlayClickEvent>,
    mut connect_ev: EventWriter<ClientMultiplayerClickEvent>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    app_exit_events.write(AppExit::Success);
                }
                MenuButtonAction::Play => {
                    info!("Starting a new game from main menu");
                    play_ev.write(ClientPlayClickEvent);
                    menu_state.set(MenuState::Main);
                }
                MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
                MenuButtonAction::SettingsDisplay => {
                    menu_state.set(MenuState::SettingsDisplay);
                }
                MenuButtonAction::SettingsSound => {
                    menu_state.set(MenuState::SettingsSound);
                }
                MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main),
                MenuButtonAction::BackToSettings => {
                    menu_state.set(MenuState::Settings);
                }
                MenuButtonAction::Multiplayer => menu_state.set(MenuState::Multiplayer),
                MenuButtonAction::MultiplayerConnect => {
                    let address = q_address.single().expect("No address input found");
                    info!("Connecting to multiplayer server at address {}", address.0);
                    connect_ev.write(ClientMultiplayerClickEvent {
                        address: address.0.clone(),
                    });
                    menu_state.set(MenuState::Main);
                }
                MenuButtonAction::SettingsName => menu_state.set(MenuState::SettingsName),
            }
        }
    }
}
