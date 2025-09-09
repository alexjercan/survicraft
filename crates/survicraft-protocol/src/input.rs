use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::input::{config::InputConfig, leafwing::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
pub enum NetworkedInput {
    #[actionlike(DualAxis)]
    Move,
}

pub fn register_input(app: &mut App) {
    app.add_plugins(InputPlugin {
        config: InputConfig::<NetworkedInput> {
            lag_compensation: true,
            ..default()
        },
    });
}
