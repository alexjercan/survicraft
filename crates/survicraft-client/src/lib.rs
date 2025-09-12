mod chat;
mod network;

use bevy::prelude::*;
use lightyear::connection::identity::is_client;
pub use network::{ClientConnection, ClientMetadata};

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum ClientStates {
    #[default]
    Connecting,
    Playing,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientPluginSet;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ClientStates>();
        app.enable_state_scoped_entities::<ClientStates>();

        app.add_plugins(network::NetworkPlugin);
        app.configure_sets(Update, network::NetworkPluginSet.in_set(ClientPluginSet));

        app.add_plugins(chat::ChatPlugin);
        app.configure_sets(Update, chat::ChatPluginSet.in_set(ClientPluginSet));

        app.add_systems(
            Update,
            (|mut state: ResMut<NextState<ClientStates>>| {
                state.set(ClientStates::Playing);
            })
            .in_set(ClientPluginSet)
            .run_if(in_state(ClientStates::Connecting).and(is_client)),
        );
        app.add_systems(
            OnEnter(ClientStates::Playing),
            setup_chat.in_set(ClientPluginSet),
        );
    }
}

fn setup_chat(mut commands: Commands) {
    commands.spawn((
        Name::new("ChatUI"),
        chat::ChatMenuRoot,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        StateScoped(ClientStates::Playing),
    ));
}
