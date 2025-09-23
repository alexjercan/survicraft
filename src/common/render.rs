use avian3d::prelude::*;
use bevy::prelude::*;
use lightyear::{
    frame_interpolation::{FrameInterpolate, FrameInterpolationPlugin},
    prelude::*,
};

pub mod prelude {
    pub use super::CommonRendererPlugin;
}

pub struct CommonRendererPlugin;

impl Plugin for CommonRendererPlugin {
    fn build(&self, app: &mut App) {
        // Set up visual interp plugins for Position/Rotation. Position/Rotation is updated in FixedUpdate
        // by the physics plugin so we make sure that in PostUpdate we interpolate it
        app.add_plugins(FrameInterpolationPlugin::<Position>::default());
        app.add_plugins(FrameInterpolationPlugin::<Rotation>::default());

        // Observers that add VisualInterpolationStatus components to entities
        // which receive a Position and are predicted
        app.add_observer(add_visual_interpolation_components);
    }
}

fn add_visual_interpolation_components(
    // We use Position because it's added by avian later, and when it's added
    // we know that Predicted is already present on the entity
    trigger: Trigger<OnAdd, Position>,
    query: Query<Entity, With<Predicted>>,
    mut commands: Commands,
) {
    if !query.contains(trigger.target()) {
        return;
    }
    commands.entity(trigger.target()).insert((
        FrameInterpolate::<Position> {
            // We must trigger change detection on visual interpolation
            // to make sure that child entities (sprites, meshes, text)
            // are also interpolated
            trigger_change_detection: true,
            ..default()
        },
        FrameInterpolate::<Rotation> {
            // We must trigger change detection on visual interpolation
            // to make sure that child entities (sprites, meshes, text)
            // are also interpolated
            trigger_change_detection: true,
            ..default()
        },
    ));
}
