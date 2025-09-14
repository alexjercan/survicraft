mod chat;
mod network;
mod terrain;
// For debugging purposes
mod controller;

use bevy::prelude::*;
use lightyear::connection::identity::is_client;
pub use network::{ClientConnection, ClientMetadata};
use survicraft_common::tilemap::prelude::*;

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

        app.add_plugins(terrain::TerrainPlugin::new(
            0,
            Vec2::splat(1.0),
            16,
            2,
            20.0,
        ));
        app.configure_sets(Update, terrain::TerrainPluginSet.in_set(ClientPluginSet));

        // For debugging purposes
        app.add_plugins(controller::WASDCameraControllerPlugin);
        app.configure_sets(
            Update,
            controller::WASDCameraControllerPluginSet.in_set(ClientPluginSet),
        );

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
        app.add_systems(
            OnEnter(ClientStates::Playing),
            create_a_single_test_chunk.in_set(ClientPluginSet),
        );
    }
}

fn setup_chat(mut commands: Commands) {
    // Temporary camera for easy testing
    commands.spawn((
        controller::WASDCameraControllerBundle::default(),
        Camera3d::default(),
        Transform::from_xyz(60.0, 60.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("RTS Camera"),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(60.0, 60.0, 00.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Directional Light"),
    ));

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

fn create_a_single_test_chunk(mut ev_discover: EventWriter<TileDiscoverEvent>) {
    ev_discover.send(TileDiscoverEvent::new(Vec2::ZERO));
}
