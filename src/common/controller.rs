use crate::{helpers::prelude::*, protocol::prelude::*};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerControllerPluginSet;

pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HeadCameraPlugin);
        app.add_plugins(InputManagerPlugin::<CameraMovement>::default());
        app.configure_sets(Update, HeadCameraSet);

        app.add_systems(
            Update,
            (
                handle_player_character,
                handle_spawn_player,
                handle_player_target,
                update_camera_input,
                update_player_input,
            )
                .in_set(PlayerControllerPluginSet),
        );
    }
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
enum CameraMovement {
    #[actionlike(DualAxis)]
    MoveAxis,
    FreeLook,
}

#[derive(Bundle, Clone, Debug)]
pub struct HeadCameraControllerBundle {
    camera: HeadCamera,
    input: HeadCameraInput,
    map: InputMap<CameraMovement>,
}

impl Default for HeadCameraControllerBundle {
    fn default() -> Self {
        Self {
            camera: HeadCamera {
                offset: Vec3::new(0.0, CHARACTER_CAPSULE_HEIGHT / 2.0, 0.0),
                ..default()
            },
            input: HeadCameraInput::default(),
            map: InputMap::default()
                .with_dual_axis(CameraMovement::MoveAxis, MouseMove::default())
                .with_dual_axis(CameraMovement::MoveAxis, GamepadStick::RIGHT)
                .with(CameraMovement::FreeLook, KeyCode::AltLeft)
                .with(CameraMovement::FreeLook, GamepadButton::LeftTrigger),
        }
    }
}

fn handle_player_character(
    mut commands: Commands,
    q_player: Query<Entity, (Added<Predicted>, With<PlayerCharacter>)>,
) {
    for entity in &q_player {
        debug!("Adding PlayerCharacterController to entity {entity:?}");
        commands.entity(entity).insert((PlayerCharacterController,));
    }
}

fn handle_player_target(
    mut commands: Commands,
    mut q_player: Query<(Entity, Has<Controlled>), (Added<Predicted>, With<PlayerCharacter>)>,
) {
    for (entity, is_controlled) in &mut q_player {
        if is_controlled {
            debug!("Adding HeadCameraTarget to controlled and predicted entity {entity:?}");
            commands.entity(entity).insert((HeadCameraTarget,));
        }
    }
}

fn handle_spawn_player(
    mut commands: Commands,
    mut q_player: Query<(Entity, Has<Controlled>), (Added<Predicted>, With<PlayerCharacter>)>,
) {
    for (entity, is_controlled) in &mut q_player {
        if is_controlled {
            debug!("Adding InputMap to controlled and predicted entity {entity:?}");
            commands.entity(entity).insert((InputMap::new([(
                CharacterAction::Jump,
                KeyCode::Space,
            )])
            .with(CharacterAction::Jump, GamepadButton::South)
            .with_dual_axis(CharacterAction::Move, GamepadStick::LEFT)
            .with_dual_axis(CharacterAction::Move, VirtualDPad::wasd())
            .with_dual_axis(CharacterAction::Look, GamepadStick::RIGHT)
            .with_dual_axis(CharacterAction::Look, MouseMove::default()),));
        } else {
            debug!("Remote character predicted for us: {entity:?}");
        }
    }
}

fn update_camera_input(mut q_camera: Query<(&mut HeadCameraInput, &ActionState<CameraMovement>)>) {
    for (mut input, action) in q_camera.iter_mut() {
        input.move_axis = action.axis_pair(&CameraMovement::MoveAxis);
    }
}

fn update_player_input(
    mut q_player: Query<
        (&mut PlayerCharacterInput, &ActionState<CharacterAction>),
        With<PlayerCharacterController>,
    >,
) {
    for (mut input, action_state) in q_player.iter_mut() {
        input.move_axis = action_state.axis_pair(&CharacterAction::Move);
        input.jump = action_state.just_pressed(&CharacterAction::Jump);
        input.look = action_state.axis_pair(&CharacterAction::Look);
    }
}
