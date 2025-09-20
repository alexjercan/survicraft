//! TODO: Document this module

use bevy::prelude::*;

/// The Head camera component, which allows for mouse look.
#[derive(Component, Clone, Copy, Debug)]
pub struct HeadCamera {
    /// The look sensitivity of the camera
    pub look_sensitivity: f32,
    /// The offset of the camera from the object it is following
    pub offset: Vec3,
    /// The minimum pitch of the camera
    pub min_pitch: f32,
    /// The maximum pitch of the camera
    pub max_pitch: f32,
}

impl Default for HeadCamera {
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
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct HeadCameraInput {
    pub move_axis: Vec2,
    pub free_look: bool,
}

/// Target for the Head camera to follow.
#[derive(Component, Clone, Copy, Debug)]
pub struct HeadCameraTarget;

pub struct HeadCameraPlugin;

impl Plugin for HeadCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_transform);
    }
}

fn sync_transform(
    mut q_camera: Query<(&mut Transform, &HeadCameraInput, &HeadCamera)>,
    target: Single<&GlobalTransform, With<HeadCameraTarget>>,
) {
    let target_transform = target.into_inner();

    for (mut transform, input, camera) in q_camera.iter_mut() {
        let pitch_delta = -input.move_axis.y * camera.look_sensitivity;
        let (_, current_pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let new_pitch = (current_pitch + pitch_delta).clamp(camera.min_pitch, camera.max_pitch);
        let (target_yaw, _, _) = target_transform.rotation().to_euler(EulerRot::YXZ);

        if input.free_look {
            let yaw_delta = -input.move_axis.x * camera.look_sensitivity;
            let new_yaw = target_yaw + yaw_delta;
            transform.rotation = Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, 0.0);
        } else {
            transform.rotation = Quat::from_euler(EulerRot::YXZ, target_yaw, new_pitch, 0.0);
        }

        transform.translation = target_transform.translation() + camera.offset;
    }
}
