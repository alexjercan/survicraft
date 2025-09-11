use bevy::prelude::*;
use std::time::Duration;

use clap::Parser;
use lightyear::prelude::{client::ClientPlugins, server::ServerPlugins};
use survicraft_protocol::FIXED_TIMESTEP_HZ;

use survicraft_launcher::{
    common::new_gui_app,
    main_menu::{
        ClientConnectEvent, ClientHostEvent, MainMenuAssets, MainMenuPlugin, MainMenuPluginSet,
        UIRoot,
    },
};

#[derive(Resource, Clone, Debug)]
enum Mode {
    Client,
    Host,
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameStates {
    #[default]
    Loading,
    MainMenu,
    Playing,
}

#[derive(Parser)]
#[command(name = "survicraft")]
#[command(version = "0.1")]
#[command(about = "Launcher for survicraft client", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();

    let mut app = new_gui_app();
    app.init_state::<GameStates>();
    app.enable_state_scoped_entities::<GameStates>();

    app.insert_resource(Mode::Client);

    app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
    });
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
    });

    app.add_plugins(survicraft_protocol::ProtocolPlugin);

    app.add_plugins(survicraft_assets::AssetsPlugin);
    app.configure_sets(
        Update,
        survicraft_assets::AssetsPluginSet.run_if(in_state(GameStates::Loading)),
    );
    app.add_systems(
        Update,
        handle_assets_loaded.run_if(in_state(GameStates::Loading)),
    );

    app.add_plugins(MainMenuPlugin);
    app.configure_sets(
        Update,
        MainMenuPluginSet.run_if(in_state(GameStates::MainMenu)),
    );
    app.add_systems(OnEnter(GameStates::MainMenu), setup_menu);

    app.add_systems(
        Update,
        (handle_play_button_pressed, handle_multiplayer_pressed)
            .run_if(in_state(GameStates::MainMenu)),
    );

    app.add_plugins(survicraft_server::ServerPlugin);
    app.configure_sets(
        Update,
        survicraft_server::ServerPluginSet
            .run_if(is_hosting.and(in_state(GameStates::Playing)))
            .before(survicraft_client::ClientPluginSet),
    );

    app.add_plugins(survicraft_client::ClientPlugin);
    app.configure_sets(
        Update,
        survicraft_client::ClientPluginSet.run_if(in_state(GameStates::Playing)),
    );

    app.run();
}

fn is_hosting(query: Option<Res<Mode>>) -> bool {
    matches!(query.map(|m| m.clone()), Some(Mode::Host))
}

fn handle_assets_loaded(
    mut commands: Commands,
    mut ev_assets: EventReader<survicraft_assets::AssetsLoadedEvent>,
    mut next_state: ResMut<NextState<GameStates>>,
    game_assets: Option<Res<survicraft_assets::GameAssets>>,
) {
    for _ in ev_assets.read() {
        info!("Assets loaded, switching to MainMenu state");

        next_state.set(GameStates::MainMenu);

        let game_assets = game_assets.as_ref().expect("GameAssets resource not found");
        commands.insert_resource(MainMenuAssets {
            exit_icon: game_assets.exit_icon.clone(),
            right_icon: game_assets.right_icon.clone(),
            wrench_icon: game_assets.wrench_icon.clone(),
        });
    }
}

fn setup_menu(mut commands: Commands) {
    commands.spawn((
        Name::new("CameraMainMenuUI"),
        Camera2d,
        // StateScoped(GameStates::MainMenu),
    ));

    commands.spawn((
        Name::new("UIRoot"),
        UIRoot,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        StateScoped(GameStates::MainMenu),
    ));
}

fn handle_play_button_pressed(
    mut ev_play: EventReader<ClientHostEvent>,
    mut next_state: ResMut<NextState<GameStates>>,
    mut mode: ResMut<Mode>,
) {
    for _ in ev_play.read() {
        next_state.set(GameStates::Playing);
        *mode = Mode::Host;
    }
}

fn handle_multiplayer_pressed(
    mut ev_connect: EventReader<ClientConnectEvent>,
    mut next_state: ResMut<NextState<GameStates>>,
    mut mode: ResMut<Mode>,
) {
    for _ in ev_connect.read() {
        next_state.set(GameStates::Playing);
        *mode = Mode::Client;
    }
}
