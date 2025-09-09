use bevy::{prelude::*, ui::FocusPolicy};
use survicraft_assets::GameAssets;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

const BORDER_COLOR_INACTIVE: Color = Color::srgb(0.25, 0.25, 0.25);
const BORDER_COLOR_ACTIVE: Color = Color::srgb(0.75, 0.52, 0.99);

const BACKGROUND_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum MenuState {
    Main,
    Settings,
    SettingsDisplay,
    SettingsSound,
    #[default]
    Disabled,
}

#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
enum DisplayQuality {
    Low,
    Medium,
    High,
}

#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
struct Volume(u32);

#[derive(Component, Clone, Copy, Debug)]
struct MainMenu;

#[derive(Component, Clone, Copy, Debug)]
struct AddressInput;

#[derive(Component, Clone, Copy, Debug)]
struct NameInput;

#[derive(Component)]
struct SelectedOption;

#[derive(Component)]
enum MenuButtonAction {
    Play,
    Settings,
    SettingsDisplay,
    SettingsSound,
    BackToMainMenu,
    BackToSettings,
    Quit,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MainMenuPluginSet;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MenuState>();
        app.enable_state_scoped_entities::<MenuState>();

        app.insert_resource(DisplayQuality::Medium);
        app.insert_resource(Volume(7));

        app.add_systems(
            OnEnter(MenuState::Main),
            main_menu_setup.in_set(MainMenuPluginSet),
        );
        app.add_systems(
            OnEnter(MenuState::Settings),
            settings_menu_setup.in_set(MainMenuPluginSet),
        );
        app.add_systems(
            OnEnter(MenuState::SettingsDisplay),
            display_settings_menu_setup.in_set(MainMenuPluginSet),
        );
        app.add_systems(
            OnEnter(MenuState::SettingsSound),
            sound_settings_menu_setup.in_set(MainMenuPluginSet),
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
            ),
        );
        app.add_systems(
            Update,
            (
                // menu_action,
                handle_button_interact,
                // handle_text_interact.before(TextInputSystem),
            )
                .in_set(MainMenuPluginSet),
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
        *background_color = match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

fn setting_button<T: Resource + Component + PartialEq + Copy>(
    interaction_query: Query<(&Interaction, &T, Entity), (Changed<Interaction>, With<Button>)>,
    selected_query: Single<(Entity, &mut BackgroundColor), With<SelectedOption>>,
    mut commands: Commands,
    mut setting: ResMut<T>,
) {
    let (previous_button, mut previous_button_color) = selected_query.into_inner();
    for (interaction, button_setting, entity) in &interaction_query {
        if *interaction == Interaction::Pressed && *setting != *button_setting {
            *previous_button_color = NORMAL_BUTTON.into();
            commands.entity(previous_button).remove::<SelectedOption>();
            commands.entity(entity).insert(SelectedOption);
            *setting = *button_setting;
        }
    }
}

fn main_menu_setup(mut commands: Commands, game_assets: Res<GameAssets>) {
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

    commands
        .spawn((
            Name::new("MainMenu"),
            MainMenu,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            StateScoped(MenuState::Main),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    // Display the game name
                    parent.spawn((
                        Text::new("Tanks"),
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
                            let icon = game_assets.right_icon.clone();
                            parent.spawn((ImageNode::new(icon), button_icon_node.clone()));
                            parent.spawn((
                                Text::new("New Game"),
                                button_text_font.clone(),
                                TextColor(TEXT_COLOR),
                            ));
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
                        // TextInput,
                        // TextInputTextFont(TextFont {
                        //     font_size: 34.,
                        //     ..default()
                        // }),
                        // TextInputTextColor(TextColor(TEXT_COLOR)),
                        // TextInputValue("127.0.0.1".to_string()),
                        // TextInputSettings {
                        //     retain_on_submit: true,
                        //     ..default()
                        // },
                        // TextInputInactive(true),
                    ));

                    parent.spawn((
                        Name::new("NameInput"),
                        NameInput,
                        Node {
                            width: Val::Px(200.0),
                            border: UiRect::all(Val::Px(5.0)),
                            padding: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        Interaction::None,
                        BorderColor(BORDER_COLOR_INACTIVE),
                        BackgroundColor(BACKGROUND_COLOR),
                        FocusPolicy::Block,
                        // TextInput,
                        // TextInputTextFont(TextFont {
                        //     font_size: 34.,
                        //     ..default()
                        // }),
                        // TextInputTextColor(TextColor(TEXT_COLOR)),
                        // TextInputValue("Player".to_string()),
                        // TextInputSettings {
                        //     retain_on_submit: true,
                        //     ..default()
                        // },
                        // TextInputInactive(true),
                    ));

                    parent
                        .spawn((
                            Name::new("SettingsButton"),
                            Button,
                            button_node.clone(),
                            BackgroundColor(NORMAL_BUTTON),
                            MenuButtonAction::Settings,
                        ))
                        .with_children(|parent| {
                            let icon = game_assets.wrench_icon.clone();
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
                            let icon = game_assets.exit_icon.clone();
                            parent.spawn((ImageNode::new(icon), button_icon_node));
                            parent.spawn((
                                Text::new("Quit"),
                                button_text_font,
                                TextColor(TEXT_COLOR),
                            ));
                        });
                });
        });
}

fn settings_menu_setup(mut commands: Commands) {
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

    commands
        .spawn((
            Name::new("SettingsMenu"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            StateScoped(MenuState::Settings),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    for (action, text) in [
                        (MenuButtonAction::SettingsDisplay, "Display"),
                        (MenuButtonAction::SettingsSound, "Sound"),
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
}

fn display_settings_menu_setup(mut commands: Commands, display_quality: Res<DisplayQuality>) {
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

    commands
        .spawn((
            Name::new("DisplaySettingsMenu"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            StateScoped(MenuState::SettingsDisplay),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    // Create a new `Node`, this time not setting its `flex_direction`. It will
                    // use the default value, `FlexDirection::Row`, from left to right.
                    parent
                        .spawn((
                            Node {
                                align_items: AlignItems::Center,
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            // Display a label for the current setting
                            parent.spawn((Text::new("Display Quality"), button_text_style.clone()));
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
}

fn sound_settings_menu_setup(mut commands: Commands, volume: Res<Volume>) {
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

    commands
        .spawn((
            Name::new("SoundSettingsMenu"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            StateScoped(MenuState::SettingsSound),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                align_items: AlignItems::Center,
                                ..default()
                            },
                        ))
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
}

// fn menu_action(
//     interaction_query: Query<
//         (&Interaction, &MenuButtonAction),
//         (Changed<Interaction>, With<Button>),
//     >,
//     mut app_exit_events: EventWriter<AppExit>,
//     mut menu_state: ResMut<NextState<MenuState>>,
//     q_address: Query<&TextInputValue, With<AddressInput>>,
//     q_name: Query<&TextInputValue, With<NameInput>>,
//     mut events: EventWriter<PlayButtonPressed>,
// ) {
//     for (interaction, menu_button_action) in &interaction_query {
//         if *interaction == Interaction::Pressed {
//             match menu_button_action {
//                 MenuButtonAction::Quit => {
//                     app_exit_events.send(AppExit::Success);
//                 }
//                 MenuButtonAction::Play => {
//                     let address = q_address.get_single().expect("AddressInput not found");
//                     let name = q_name.get_single().expect("NameInput not found");
//
//                     events.send(PlayButtonPressed {
//                         address: address.0.clone(),
//                         name: name.0.clone(),
//                     });
//
//                     menu_state.set(MenuState::Disabled);
//                 }
//                 MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
//                 MenuButtonAction::SettingsDisplay => {
//                     menu_state.set(MenuState::SettingsDisplay);
//                 }
//                 MenuButtonAction::SettingsSound => {
//                     menu_state.set(MenuState::SettingsSound);
//                 }
//                 MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main),
//                 MenuButtonAction::BackToSettings => {
//                     menu_state.set(MenuState::Settings);
//                 }
//             }
//         }
//     }
// }
