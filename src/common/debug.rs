use crate::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::{input::InputBuffer, server::ClientOf, *};

pub struct LoggingDebugPlugin;

impl Plugin for LoggingDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Last, last_log);
        app.add_systems(FixedLast, fixed_last_log);
    }
}

pub(crate) fn fixed_last_log(
    timeline: Single<(&LocalTimeline, Has<Rollback>), Or<(With<Client>, Without<ClientOf>)>>,
    players: Query<
        (
            Entity,
            &Position,
            Option<&VisualCorrection<Position>>,
            Option<&ActionState<CharacterAction>>,
            Option<&InputBuffer<ActionState<CharacterAction>>>,
        ),
        (With<PlayerCharacter>, Without<Confirmed>),
    >,
) {
    let (timeline, rollback) = timeline.into_inner();
    let tick = timeline.tick();

    for (entity, position, correction, action_state, input_buffer) in players.iter() {
        let pressed = action_state.map(|a| a.axis_pair(&CharacterAction::Move));
        let last_buffer_tick = input_buffer.and_then(|b| b.get_last_with_tick().map(|(t, _)| t));
        info!(
            ?rollback,
            ?tick,
            ?entity,
            ?position,
            ?correction,
            ?pressed,
            ?last_buffer_tick,
            "Player - FixedLast"
        );
    }
}

pub(crate) fn last_log(
    timeline: Single<(&LocalTimeline, Has<Rollback>), Or<(With<Client>, Without<ClientOf>)>>,
    players: Query<
        (
            Entity,
            &Position,
            &Transform,
            // Option<&FrameInterpolate<Position>>,
            Option<&VisualCorrection<Position>>,
        ),
        (With<PlayerCharacter>, Without<Confirmed>),
    >,
) {
    let (timeline, rollback) = timeline.into_inner();
    let tick = timeline.tick();

    for (entity, position, transform /*, interpolate*/, correction) in players.iter() {
        info!(
            ?rollback,
            ?tick,
            ?entity,
            ?position,
            ?transform,
            // ?interpolate,
            ?correction,
            "Player - Last"
        );
    }
}
