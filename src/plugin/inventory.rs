use bevy::prelude::*;

use crate::common::prelude::*;
use super::controller::*;

pub(super) struct GameInventoryPlugin {
    pub render: bool,
}

impl Plugin for GameInventoryPlugin {
    // TODO: How to make this network proof?
    fn build(&self, app: &mut App) {
        app.add_plugins(InventoryPlugin);

        if self.render {
            app.add_plugins(InventoryRenderPlugin);
        }

        app.add_systems(Update, testing_spawn_item);
        app.add_systems(Update, test_f_input);
    }
}

// NOTE: These are for debug/testing purposes only

// testing system:
// - press Z will spawn 1 wood item at the player position
fn testing_spawn_item(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<PlayerController>>,
    item_assets: Res<ItemAssets>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::KeyZ) {
        for transform in player_query.iter() {
            let Some(wood_item) = item_assets.get_item(&"wood".to_string()) else {
                error!("No wood item found");
                return;
            };

            commands.spawn((
                Name::new("Item"),
                Item(wood_item.id.clone()),
                Transform::from_translation(transform.translation + transform.forward().xz().extend(0.0).xzy() * 3.0),
            ));
        }
    }
}

// testing system:
// press F will set the craft input to true for 1 frame
fn test_f_input(mut query: Query<&mut CrafterInput>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    for mut input in &mut query {
        input.craft = keyboard_input.just_pressed(KeyCode::KeyF);
    }
}
