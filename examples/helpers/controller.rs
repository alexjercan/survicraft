use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};
use survicraft::prelude::*;

pub struct PlayerControllerPlugin {
    pub dynamic: bool,
    pub render: bool,
}

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        if self.dynamic {
            app.add_plugins(PhysicsCharacterPlugin);
        } else {
            app.add_plugins(KinematicCharacterPlugin);
        }
        app.add_plugins(HeadControllerPlugin);

        app.add_plugins(InputManagerPlugin::<CharacterAction>::default());
        app.add_plugins(InputManagerPlugin::<HeadAction>::default());

        if self.render {
            app.add_plugins(PlayerRenderPlugin);
        }

        app.add_observer(on_add_player_controller);

        app.add_systems(
            Update,
            (
                update_character_input,
                update_head_input,
                sync_character_rotation,
            ),
        );
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
pub enum CharacterAction {
    #[actionlike(DualAxis)]
    Move,
    Jump,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
pub enum HeadAction {
    #[actionlike(DualAxis)]
    Look,
}

fn on_add_player_controller(
    trigger: Trigger<OnAdd, super::PlayerController>,
    mut commands: Commands,
) {
    let entity = trigger.target();

    commands.spawn((
        Name::new("Head"),
        InputMap::default()
            .with_dual_axis(HeadAction::Look, GamepadStick::RIGHT)
            .with_dual_axis(HeadAction::Look, MouseMove::default()),
        HeadController {
            offset: Vec3::new(0.0, CHARACTER_CAPSULE_HEIGHT / 2.0, 0.0),
            ..default()
        },
        HeadControllerInput::default(),
        HeadControllerTarget(entity),
        Camera3d::default(),
        Transform::default(),
        Rotation::default(),
    ));

    commands
        .entity(entity)
        .insert((InputMap::new([(CharacterAction::Jump, KeyCode::Space)])
            .with(CharacterAction::Jump, GamepadButton::South)
            .with_dual_axis(CharacterAction::Move, GamepadStick::LEFT)
            .with_dual_axis(CharacterAction::Move, VirtualDPad::wasd()),));
}

fn update_character_input(
    mut q_player: Query<(&mut CharacterInput, &ActionState<CharacterAction>)>,
) {
    for (mut input, action_state) in q_player.iter_mut() {
        input.move_axis = action_state.axis_pair(&CharacterAction::Move);
        input.jump = action_state.just_pressed(&CharacterAction::Jump);
    }
}

fn update_head_input(mut q_head: Query<(&mut HeadControllerInput, &ActionState<HeadAction>)>) {
    for (mut input, action_state) in q_head.iter_mut() {
        input.look_axis = action_state.axis_pair(&HeadAction::Look);
    }
}

fn sync_character_rotation(
    mut q_player: Query<&mut Rotation, With<super::PlayerController>>,
    q_head: Query<(&Transform, &HeadControllerTarget), With<HeadController>>,
) {
    for (transform, &HeadControllerTarget(target)) in q_head.iter() {
        let mut rotation = match q_player.get_mut(target) {
            Ok(r) => r,
            Err(_) => {
                warn!("HeadControllerTarget entity {target:?} does not have a Rotation");
                continue;
            }
        };

        let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
        rotation.0 = Quat::from_euler(EulerRot::YXZ, yaw, 0.0, 0.0);
    }
}

struct PlayerRenderPlugin;

impl Plugin for PlayerRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_render_player);
    }
}

fn handle_render_player(
    q_player: Query<Entity, Added<super::PlayerController>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in q_player.iter() {
        debug!("Rendering player entity {entity:?}");

        commands.entity(entity).insert((
            Mesh3d(meshes.add(Mesh::from(Capsule3d {
                radius: CHARACTER_CAPSULE_RADIUS,
                half_length: CHARACTER_CAPSULE_HEIGHT / 2.0,
                ..default()
            }))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.7, 0.6),
                ..default()
            })),
        ));
    }
}
