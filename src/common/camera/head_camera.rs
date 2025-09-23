//! TODO: Document this module

// NOTE: I know... using avian3d just for Rotation... but it is convenient
use avian3d::prelude::*;
use bevy::prelude::*;

/// The Head camera component, which allows for mouse look.
#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct HeadController {
    /// The look sensitivity of the camera
    pub look_sensitivity: f32,
    /// The offset of the camera from the object it is following
    pub offset: Vec3,
    /// The minimum pitch of the camera
    pub min_pitch: f32,
    /// The maximum pitch of the camera
    pub max_pitch: f32,
}

impl Default for HeadController {
    fn default() -> Self {
        Self {
            look_sensitivity: 0.0025,
            offset: Vec3::new(0.0, 1.5, 0.0),
            min_pitch: std::f32::consts::FRAC_PI_2 * -0.99,
            max_pitch: std::f32::consts::FRAC_PI_2 * 0.99,
        }
    }
}

/// The input component for the Head camera, which stores the current input state.
/// This component should be updated by user input systems to control the camera.
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
pub struct HeadControllerInput {
    pub look_axis: Vec2,
}

/// Target for the Head camera to follow.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Reflect)]
pub struct HeadControllerTarget(pub Entity);

pub struct HeadControllerPlugin;

impl Plugin for HeadControllerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HeadController>()
            .register_type::<HeadControllerInput>()
            .register_type::<HeadControllerTarget>();

        app.add_systems(Update, sync_transform);
    }
}

fn sync_transform(
    mut q_camera: Query<(
        &mut Transform,
        &mut Rotation,
        &HeadControllerInput,
        &HeadController,
        &HeadControllerTarget,
    )>,
    q_target: Query<&GlobalTransform, Without<HeadController>>,
) {
    for (mut transform, mut rotation, input, camera, &HeadControllerTarget(target)) in
        q_camera.iter_mut()
    {
        let target_transform = match q_target.get(target) {
            Ok(t) => t,
            Err(_) => {
                error!("HeadControllerTarget entity {target:?} does not have a GlobalTransform");
                continue;
            }
        };

        let pitch_delta = -input.look_axis.y * camera.look_sensitivity;
        let (_, current_pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let new_pitch = (current_pitch + pitch_delta).clamp(camera.min_pitch, camera.max_pitch);

        let yaw_delta = -input.look_axis.x * camera.look_sensitivity;
        let (target_yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let new_yaw = target_yaw + yaw_delta;

        rotation.0 = Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, 0.0);

        transform.rotation = rotation.0;
        transform.translation = target_transform.translation() + camera.offset;
    }
}
