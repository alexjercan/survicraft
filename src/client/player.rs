//! The player plugin handles the client side player logic.

use crate::protocol::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PlayerPluginSet;

pub(crate) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_spawn_player.in_set(PlayerPluginSet));
    }
}

fn handle_spawn_player(
    mut commands: Commands,
    mut q_player: Query<(Entity, Has<Controlled>), (Added<Predicted>, With<PlayerCharacter>)>,
) {
    for (entity, is_controlled) in &mut q_player {
        if is_controlled {
            debug!("Adding InputMap to controlled and predicted entity {entity:?}");
            commands.entity(entity).insert(
                InputMap::new([(CharacterAction::Jump, KeyCode::Space)])
                    .with(CharacterAction::Jump, GamepadButton::South)
                    .with_dual_axis(CharacterAction::Move, GamepadStick::LEFT)
                    .with_dual_axis(CharacterAction::Move, VirtualDPad::wasd()),
            );
        } else {
            debug!("Remote character predicted for us: {entity:?}");
        }
    }
}
