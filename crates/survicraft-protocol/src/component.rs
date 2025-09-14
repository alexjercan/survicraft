use avian3d::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Reflect)]
pub struct PlayerId(pub PeerId);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct PlayerName(pub String);

pub fn register_components(app: &mut App) {
    app.register_type::<PlayerId>()
        .register_type::<PlayerName>();

    app.register_component::<Name>()
        .add_prediction(PredictionMode::Once)
        .add_interpolation(InterpolationMode::Once);

    app.register_component::<PlayerId>()
        .add_prediction(PredictionMode::Once)
        .add_interpolation(InterpolationMode::Once);

    app.register_component::<PlayerName>()
        .add_prediction(PredictionMode::Once)
        .add_interpolation(InterpolationMode::Once);

    // Fully replicated, but not visual, so no need for lerp/corrections:
    app.register_component::<LinearVelocity>()
        .add_prediction(PredictionMode::Full);

    app.register_component::<AngularVelocity>()
        .add_prediction(PredictionMode::Full);

    app.register_component::<ExternalForce>()
        .add_prediction(PredictionMode::Full);

    app.register_component::<ExternalImpulse>()
        .add_prediction(PredictionMode::Full);

    app.register_component::<ComputedMass>()
        .add_prediction(PredictionMode::Full);

    // Position and Rotation have a `correction_fn` set, which is used to smear rollback errors
    // over a few frames, just for the rendering part in postudpate.
    //
    // They also set `interpolation_fn` which is used by the VisualInterpolationPlugin to smooth
    // out rendering between fixedupdate ticks.
    app.register_component::<Position>()
        .add_prediction(PredictionMode::Full)
        .add_should_rollback(position_should_rollback)
        .add_linear_correction_fn()
        .add_interpolation(InterpolationMode::Full)
        .add_linear_interpolation_fn();

    app.register_component::<Rotation>()
        .add_prediction(PredictionMode::Full)
        .add_should_rollback(rotation_should_rollback)
        .add_linear_correction_fn()
        .add_interpolation(InterpolationMode::Full)
        .add_linear_interpolation_fn();
}


fn position_should_rollback(this: &Position, that: &Position) -> bool {
    (this.0 - that.0).length() >= 0.01
}

fn rotation_should_rollback(this: &Rotation, that: &Rotation) -> bool {
    this.angle_between(that.0) >= 0.01
}
