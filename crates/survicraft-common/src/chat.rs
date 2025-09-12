use bevy::{prelude::*, ui::FocusPolicy};
use bevy_simple_text_input::*;

const BORDER_COLOR: Color = Color::srgba(0.25, 0.25, 0.25, 0.25);
const BACKGROUND_COLOR: Color = Color::srgba(0.15, 0.15, 0.15, 0.75);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

/// Marker component for the UI root node.
/// Add this component to an entity to make it the root of the UI and spawn the chat UI as a child.
#[derive(Component)]
pub struct ChatMenuRoot;

/// Event get's triggered when a chat message is submitted.
#[derive(Debug, Clone, Event, Deref, DerefMut)]
pub struct ChatMessageEvent(pub String);

#[derive(Resource, Clone, Debug, Default, PartialEq, Eq, Deref, DerefMut)]
pub struct ChatEnabled(pub bool);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChatPluginSet;

#[derive(Component, Clone, Copy, Debug)]
struct ChatUI;

#[derive(Component, Clone, Copy, Debug)]
struct ChatMessageInput;

pub struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChatEnabled>();
        app.add_event::<ChatMessageEvent>();

        app.add_systems(
            Update,
            (chat_ui_setup, on_chat_message)
                .in_set(ChatPluginSet)
                .run_if(resource_equals(ChatEnabled(false))),
        );
    }
}

fn chat_ui_setup(mut commands: Commands, root: Single<Entity, (With<ChatMenuRoot>, Added<ChatMenuRoot>)>) {
    commands.entity(root.entity()).with_children(|parent| {
        parent
            .spawn((
                Name::new("Chat UI"),
                ChatUI,
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
                parent.spawn((
                    Name::new("ChatMessageInput"),
                    ChatMessageInput,
                    Node {
                        width: Val::Px(500.0),
                        border: UiRect::all(Val::Px(5.0)),
                        padding: UiRect::all(Val::Px(5.0)),
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
                    TextInputInactive(false),
                ));
            });
    });
}

fn on_chat_message(
    mut ev_chat: EventWriter<ChatMessageEvent>,
    mut ev_input: EventReader<TextInputSubmitEvent>,
    message_input: Single<Entity, With<ChatMessageInput>>,
) {
    for ev in ev_input.read() {
        if ev.entity != message_input.entity() {
            continue;
        }

        let msg = ev.value.trim();
        if !msg.is_empty() {
            ev_chat.write(ChatMessageEvent(msg.to_string()));
        }
    }
}
