//! The Chat plugin provides a simple chat UI and handles sending and receiving chat messages.

use std::collections::VecDeque;

use crate::protocol::prelude::*;
use bevy::{prelude::*, ui::FocusPolicy};
use bevy_simple_text_input::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;

const BORDER_COLOR: Color = Color::srgba(0.25, 0.25, 0.25, 0.25);
const BACKGROUND_COLOR: Color = Color::srgba(0.15, 0.15, 0.15, 0.75);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

// TODO: Maybe move this in lib
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
enum ChatInputAction {
    OpenChat,
    CloseChat,
}

impl ChatInputAction {
    fn default_input_map() -> InputMap<Self> {
        InputMap::default()
            .with(Self::OpenChat, KeyCode::Enter)
            .with(Self::CloseChat, KeyCode::Escape)
    }
}

#[derive(Component)]
pub(crate) struct ChatMenuRoot;

#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Deref, DerefMut)]
pub struct ChatEnabled(pub bool);

#[derive(Debug, Component, Clone, PartialEq, Eq)]
pub struct HistoryListUI {
    pub messages: VecDeque<Entity>,
    pub max_messages: usize,
}

#[derive(Debug, Component, Clone, Copy, PartialEq, Eq)]
struct HistoryItemUI;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ChatPluginSet;

#[derive(Component, Clone, Copy, Debug)]
struct ChatMessageInput;

pub(crate) struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChatEnabled>();
        app.add_plugins(InputManagerPlugin::<ChatInputAction>::default());

        app.add_systems(
            Update,
            (
                chat_ui_setup,
                on_chat_message,
                (
                    on_chat_submit.run_if(resource_equals(ChatEnabled(true))),
                    handle_chat_input_action,
                )
                    .chain(),
            )
                .in_set(ChatPluginSet),
        );
    }
}

fn chat_ui_setup(
    mut commands: Commands,
    root: Single<Entity, (With<ChatMenuRoot>, Added<ChatMenuRoot>)>,
) {
    println!("Setting up chat UI");

    commands.entity(root.entity()).with_children(|parent| {
        parent.spawn((
            Name::new("ChatInputManager"),
            ChatInputAction::default_input_map(),
        ));

        parent
            .spawn((
                Name::new("ChatUI"),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Relative, // default, but explicit
                    ..default()
                },
            ))
            .with_children(|parent| {
                // --- Chat history in bottom-left ---
                parent.spawn((
                    Name::new("ChatHistoryUI"),
                    HistoryListUI {
                        messages: VecDeque::new(),
                        max_messages: 5,
                    },
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

                // --- Input box centered ---
                parent
                    .spawn((
                        Name::new("ChatInputUI"),
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            position_type: PositionType::Relative,
                            ..default()
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Name::new("ChatMessageInput"),
                            ChatMessageInput,
                            Node {
                                width: Val::Px(500.0),
                                height: Val::Px(64.0),
                                border: UiRect::all(Val::Px(5.0)),
                                padding: UiRect::all(Val::Px(5.0)),
                                position_type: PositionType::Absolute,
                                left: Val::Percent(50.0),
                                top: Val::Percent(50.0),
                                margin: UiRect {
                                    left: Val::Px(-250.0), // half width
                                    top: Val::Px(-34.0),   // half height
                                    ..default()
                                },
                                ..default()
                            },
                            Interaction::None,
                            BorderColor(BORDER_COLOR),
                            BackgroundColor(BACKGROUND_COLOR),
                            FocusPolicy::Block,
                            TextInput,
                            TextInputTextFont(TextFont {
                                font_size: 34.,
                                ..default()
                            }),
                            TextInputTextColor(TextColor(TEXT_COLOR)),
                            TextInputValue("".to_string()),
                            TextInputSettings {
                                retain_on_submit: false,
                                ..default()
                            },
                            TextInputInactive(true),
                            Visibility::Hidden,
                        ));
                    });
            });
    });
}

fn on_chat_submit(
    mut ev_input: EventReader<TextInputSubmitEvent>,
    message_input: Single<Entity, With<ChatMessageInput>>,
    mut sender: Single<&mut MessageSender<ClientChatMessage>>,
) {
    for ev in ev_input.read() {
        if ev.entity != message_input.entity() {
            continue;
        }

        let msg = ev.value.trim();
        if !msg.is_empty() {
            sender.send::<MessageChannel>(ClientChatMessage {
                message: msg.to_string(),
            });
        }
    }
}

fn create_chat_history_item(commands: &mut Commands, sender: &str, message: &str) -> Entity {
    commands
        .spawn((
            Name::new("ChatHistoryItem"),
            HistoryItemUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Auto,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("ChatHistoryItemText"),
                Text::new(format!("{}: {}", sender, message)),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
            ));
        })
        .id()
}

fn on_chat_message(
    mut commands: Commands,
    mut receiver: Single<&mut MessageReceiver<ServerChatMessage>>,
    q_players: Query<(&PlayerName, &PlayerId), With<Replicated>>,
    history: Single<(Entity, &mut HistoryListUI)>,
) {
    let (entity, mut history) = history.into_inner();

    for message in receiver.receive() {
        if let Some((name, _)) = q_players.iter().find(|(_, id)| id.0 == message.sender) {
            let item = create_chat_history_item(&mut commands, &name.0, &message.message);
            commands.entity(entity).add_child(item);
            history.messages.push_back(item);

            if history.messages.len() > history.max_messages {
                if let Some(old) = history.messages.pop_front() {
                    commands.entity(old).despawn();
                }
            }
        } else {
            warn!(
                "Received chat message from unknown player ID {:?}",
                message.sender
            );
        }
    }
}

fn handle_chat_input_action(
    action_state: Single<&ActionState<ChatInputAction>>,
    mut chat_enabled: ResMut<ChatEnabled>,
    input: Single<(&mut Visibility, &mut TextInputInactive), With<ChatMessageInput>>,
) {
    let (mut visibility, mut text_input) = input.into_inner();

    if action_state.just_pressed(&ChatInputAction::OpenChat) {
        chat_enabled.0 = true;
        *visibility = Visibility::Visible;
        text_input.0 = false;
    } else if action_state.just_pressed(&ChatInputAction::CloseChat) {
        chat_enabled.0 = false;
        *visibility = Visibility::Hidden;
        text_input.0 = true;
    }
}
