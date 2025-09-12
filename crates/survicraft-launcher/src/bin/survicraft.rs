use bevy::prelude::*;
use bevy_simple_text_input::TextInputPlugin;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    time::Duration,
};
use survicraft_client::ClientConnection;
use survicraft_common::{
    chat::ChatMenuRoot,
    debug::{DebugPlugin, DebugPluginSet},
    main_menu::{
        ClientMultiplayerClickEvent, ClientPlayClickEvent, MainMenuAssets, MainMenuPlugin,
        MainMenuPluginSet, MainMenuRoot,
    },
};
use survicraft_server::ServerListener;

use clap::Parser;
use lightyear::prelude::{client::ClientPlugins, server::ServerPlugins};
use survicraft_protocol::{FIXED_TIMESTEP_HZ, SERVER_ADDR, SERVER_PORT};

use survicraft_launcher::common::new_gui_app;

#[derive(Resource, Clone, Debug, PartialEq, Eq)]
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

    app.add_plugins(DebugPlugin);
    app.configure_sets(Update, DebugPluginSet);

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

    app.add_plugins(TextInputPlugin);

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
            .run_if(resource_equals(Mode::Host).and(in_state(GameStates::Playing)))
            .before(survicraft_client::ClientPluginSet),
    );

    app.add_plugins(survicraft_client::ClientPlugin);
    app.configure_sets(
        Update,
        survicraft_client::ClientPluginSet.run_if(in_state(GameStates::Playing)),
    );

    app.add_systems(OnEnter(GameStates::Playing), setup_game);

    app.run();
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
        Name::new("MainMenuUI"),
        MainMenuRoot,
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
    mut commands: Commands,
    mut ev_play: EventReader<ClientPlayClickEvent>,
    mut next_state: ResMut<NextState<GameStates>>,
    mut mode: ResMut<Mode>,
) {
    for _ in ev_play.read() {
        next_state.set(GameStates::Playing);
        *mode = Mode::Host;

        commands.spawn((
            Name::new("ServerListener"),
            ServerListener,
            StateScoped(GameStates::Playing),
        ));

        commands.spawn((
            Name::new("ClientConnection"),
            ClientConnection {
                address: SERVER_ADDR,
            },
            StateScoped(GameStates::Playing),
        ));
    }
}

fn handle_multiplayer_pressed(
    mut commands: Commands,
    mut ev_multiplayer: EventReader<ClientMultiplayerClickEvent>,
    mut next_state: ResMut<NextState<GameStates>>,
    mut mode: ResMut<Mode>,
) {
    for event in ev_multiplayer.read() {
        next_state.set(GameStates::Playing);
        *mode = Mode::Client;
        let addr = IpAddr::from_str(&event.address).unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));
        commands.spawn((
            Name::new("ClientConnection"),
            ClientConnection {
                address: SocketAddr::new(addr, SERVER_PORT),
            },
            StateScoped(GameStates::Playing),
        ));
    }
}

fn setup_game(mut commands: Commands) {
    // commands.spawn((
    //     Name::new("Camera3d"),
    //     Camera3d::default(),
    //     StateScoped(GameStates::Playing),
    // ));

    commands.spawn((
        Name::new("ChatUI"),
        ChatMenuRoot,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        StateScoped(GameStates::Playing),
    ));
}
