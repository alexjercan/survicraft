//! The Status plugin provides a simple status bar UI for displaying different metrics.
//! The idea is it should be easy to add what metrics to display via generic components
//! Then you update the components and the ststaus bar updates automatically.

use bevy::prelude::*;

pub mod prelude {
    pub use super::{StatusBarItem, StatusBarPlugin, StatusBarRoot};
}

/// The StatusBarRoot component is a marker component that indicates the root node of the status
/// bar UI.
#[derive(Component)]
pub struct StatusBarRoot;

#[derive(Component, Debug, Clone, PartialEq, Reflect)]
pub struct StatusBarItem {
    pub icon: Option<Handle<Image>>,
    pub value: u32,
    pub label: String,
    pub mapping: Vec<(Option<u32>, Color)>,
}

pub struct StatusBarPlugin;

impl Plugin for StatusBarPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StatusBarItem>();

        app.add_systems(Update, setup_status_bar);
        app.add_systems(Update, update_status_bar);
    }
}

fn setup_status_bar(
    mut commands: Commands,
    root: Single<Entity, (With<StatusBarRoot>, Added<StatusBarRoot>)>,
    q_items: Query<(Entity, &StatusBarItem)>,
) {
    debug!("Setting up status bar UI...");

    commands.entity(root.entity()).with_children(|parent| {
        for (entity, item) in &q_items {
            parent
                .spawn((
                    Name::new(format!("StatusBarItem: {}", item.label)),
                    Node {
                        width: Val::Auto,
                        height: Val::Px(24.0),
                        margin: UiRect::all(Val::Px(4.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(4.0),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    if let Some(icon) = &item.icon {
                        parent.spawn((
                            Name::new("StatusBarItemIcon"),
                            ImageNode {
                                image: icon.clone(),
                                ..default()
                            },
                            Node {
                                width: Val::Px(16.0),
                                height: Val::Px(16.0),
                                ..default()
                            },
                        ));
                    }
                    parent.spawn((
                        Name::new("StatusBarItemValue"),
                        StatusBarItemValue(entity),
                        Text::new(item.value.to_string().clone()),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                    parent.spawn((
                        Name::new("StatusBarItemLabel"),
                        Text::new(item.label.clone()),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                    ));
                });
        }
    });
}

#[derive(Component, Debug, Clone, PartialEq, Deref, DerefMut)]
struct StatusBarItemValue(Entity);

fn update_status_bar(
    q_items: Query<&StatusBarItem>,
    mut q_text: Query<(&mut Text, &mut TextColor, &StatusBarItemValue)>,
) {
    for (mut text, mut color, StatusBarItemValue(entity)) in &mut q_text {
        if let Ok(item) = q_items.get(*entity) {
            **text = item.value.to_string();

            // Update color based on mapping
            let mut new_color = Color::WHITE;
            for (threshold, map_color) in &item.mapping {
                if let Some(thresh) = threshold {
                    if item.value <= *thresh {
                        new_color = *map_color;
                        break;
                    }
                } else {
                    new_color = *map_color;
                }
            }
            **color = new_color;
        }
    }
}
