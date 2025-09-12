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
}
