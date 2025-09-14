use bevy::prelude::*;
use bevy_simple_text_input::TextInputPlugin;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    time::Duration,
};
use survicraft_assets::prelude::*;
use survicraft_client::{ClientConnection, ClientMetadata};
use survicraft_common::{
    PlayerNameSetting,
    main_menu::{
        ClientMultiplayerClickEvent, ClientPlayClickEvent, MainMenuIcons, MainMenuPlugin,
        MainMenuPluginSet, MainMenuRoot,
    },
    terrain::prelude::*,
};
use survicraft_server::ServerListener;

use clap::Parser;
use lightyear::prelude::{client::ClientPlugins, server::ServerPlugins};
use survicraft_protocol::{FIXED_TIMESTEP_HZ, SERVER_ADDR, SERVER_PORT};

use survicraft_common::setup::new_gui_app;

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

    app.add_plugins(survicraft_protocol::ProtocolPlugin);

    app.add_plugins(AssetsPlugin);
    app.configure_sets(
        Update,
        AssetsPluginSet.run_if(in_state(GameStates::Loading)),
    );
    app.add_systems(
        Update,
        handle_assets_loaded
            .run_if(in_state(GameStates::Loading).and(resource_added::<MainMenuAssets>)),
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

    // Maybe will have a `common` plugin later
    app.add_plugins(TerrainPlugin::default().with_seed(0));
    app.configure_sets(
        Update,
        TerrainPluginSet.run_if(in_state(GameStates::Playing)),
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

    app.run();
}

fn handle_assets_loaded(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameStates>>,
    assets: Res<MainMenuAssets>,
) {
    info!("Assets loaded, switching to MainMenu state");

    next_state.set(GameStates::MainMenu);

    commands.insert_resource(MainMenuIcons {
        exit_icon: assets.exit_icon.clone(),
        right_icon: assets.right_icon.clone(),
        wrench_icon: assets.wrench_icon.clone(),
    });

    // TODO: I want to load these from file, but for now, hardcode them
    // with some kind of cool syntax like:
    //
    // [terrain]
    //     id="deep_water"
    //     name="Deep Water"
    //     [generation]
    //         elevation_min=None
    //         elevation_max=0.25
    //     [/generation]
    // [/terrain]
    commands.insert_resource(TerrainAssets::new(vec![
        TileAsset {
            id: "deep_water".to_string(),
            name: "Deep Water".to_string(),
            generation: TileGeneration {
                elevation_min: None,
                elevation_max: Some(0.25),
            },
        },
        TileAsset {
            id: "shallow_water".to_string(),
            name: "Shallow Water".to_string(),
            generation: TileGeneration {
                elevation_min: Some(0.25),
                elevation_max: Some(0.5),
            },
        },
        TileAsset {
            id: "sand".to_string(),
            name: "Sand".to_string(),
            generation: TileGeneration {
                elevation_min: Some(0.5),
                elevation_max: Some(0.55),
            },
        },
        TileAsset {
            id: "grass".to_string(),
            name: "Grass".to_string(),
            generation: TileGeneration {
                elevation_min: Some(0.55),
                elevation_max: Some(0.75),
            },
        },
        TileAsset {
            id: "hills".to_string(),
            name: "Hills".to_string(),
            generation: TileGeneration {
                elevation_min: Some(0.75),
                elevation_max: Some(0.9),
            },
        },
        TileAsset {
            id: "mountain".to_string(),
            name: "Mountain".to_string(),
            generation: TileGeneration {
                elevation_min: Some(0.9),
                elevation_max: None,
            },
        },
    ]));
}

fn setup_menu(mut commands: Commands) {
    commands.spawn((
        Name::new("CameraMainMenuUI"),
        Camera2d,
        StateScoped(GameStates::MainMenu),
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
    player_name: Res<PlayerNameSetting>,
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
            ClientMetadata {
                username: (**player_name).clone(),
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
    player_name: Res<PlayerNameSetting>,
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
            ClientMetadata {
                username: (**player_name).clone(),
            },
            StateScoped(GameStates::Playing),
        ));
    }
}
