use crate::{helpers::prelude::*, protocol::prelude::*};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;

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

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HeadCameraControllerPluginSet;

pub struct HeadCameraControllerPlugin;

impl Plugin for HeadCameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HeadCameraPlugin);
        app.add_plugins(InputManagerPlugin::<CameraMovement>::default());
        app.configure_sets(Update, HeadCameraSet);

        app.add_observer(add_player_character);
        app.add_systems(
            Update,
            (
                handle_player_target,
                update_camera_input,
                update_player_input,
            )
                .in_set(HeadCameraControllerPluginSet),
        );
    }
}

fn add_player_character(
    trigger: Trigger<OnAdd, PlayerCharacter>,
    q_player: Query<Entity, (With<PlayerCharacter>, Without<Replicated>)>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    if !q_player.contains(entity) {
        return;
    }

    info!("Adding PlayerCharacterController to entity {entity:?}");
    commands.entity(entity).insert(PlayerCharacterController);
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
