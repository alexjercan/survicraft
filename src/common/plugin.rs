//! A Bevy plugin that serves as the main entry point for the game launcher.

use super::{assets::*, main_menu::*};
use crate::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_simple_text_input::TextInputPlugin;
use lightyear::{
    connection::identity::is_server,
    prelude::{client::ClientPlugins, server::ServerPlugins},
};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    time::Duration,
};

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum LauncherStates {
    #[default]
    Loading,
    MainMenu,
    Playing,
}

pub struct LauncherPlugin;

impl Plugin for LauncherPlugin {
    fn build(&self, app: &mut App) {
        // Initialize the state machine
        app.init_state::<LauncherStates>();
        app.enable_state_scoped_entities::<LauncherStates>();

        // Multiplayer setup for lightyear. Here we set up both client and server plugins,
        // but we will control which ones are active using states and conditions. In case
        // the player is hosting, both client and server will be active. In case the player
        // is joining, only the client will be active.
        app.add_plugins(ClientPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        });
        app.add_plugins(ServerPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        });

        // Protocol plugin for handling message serialization and deserialization.
        app.add_plugins(ProtocolPlugin);

        // Asset loading. This will transition from Loading to MainMenu state once done.
        app.add_systems(OnEnter(LauncherStates::Loading), setup_terrain_assets);
        app.add_loading_state(
            LoadingState::new(LauncherStates::Loading)
                .continue_to_state(LauncherStates::MainMenu)
                // Load assets for the main menu
                .load_collection::<MainMenuAssets>(),
        );

        // Utility plugins. Text input plugin for handling text fields.
        app.add_plugins(TextInputPlugin);

        // Main Menu setup and event handling for starting the game.
        app.add_systems(OnEnter(LauncherStates::MainMenu), setup_menu);
        app.add_plugins(MainMenuPlugin);
        app.configure_sets(
            Update,
            MainMenuPluginSet.run_if(in_state(LauncherStates::MainMenu)),
        );
        app.add_systems(
            Update,
            (handle_play_button_pressed, handle_multiplayer_pressed)
                .run_if(in_state(LauncherStates::MainMenu)),
        );

        // --- Playing related stuff below here ---

        // Physics setup. We disable interpolation and sleeping to ensure consistent physics
        app.add_plugins(
            PhysicsPlugins::default()
                .build()
                .disable::<PhysicsInterpolationPlugin>()
                // disable Sleeping plugin as it can mess up physics rollbacks
                .disable::<SleepingPlugin>(),
        );

        // Chat setup. We set up chat UI and related systems.
        app.add_systems(OnEnter(LauncherStates::Playing), setup_chat);
        app.add_plugins(ChatPlugin);
        app.configure_sets(
            Update,
            ChatPluginSet.run_if(in_state(LauncherStates::Playing)),
        );

        // Terrain setup. We set up terrain assets and the terrain plugin itself.
        // This will run only in the Playing state.
        app.add_plugins(TerrainPlugin::default().with_seed(0));
        app.configure_sets(
            Update,
            TerrainPluginSet.run_if(in_state(LauncherStates::Playing)),
        );
        app.add_plugins(TerrainRenderPlugin::default());
        app.configure_sets(
            Update,
            TerrainRenderPluginSet.run_if(in_state(LauncherStates::Playing)),
        );

        // Player setup. We set up player-related systems and the player plugin.
        app.add_plugins(PlayerPlugin);
        app.configure_sets(
            FixedUpdate,
            PlayerPluginSet.run_if(in_state(LauncherStates::Playing)),
        );
        app.configure_sets(
            Update,
            PlayerPluginSet.run_if(in_state(LauncherStates::Playing)),
        );
        app.add_plugins(PlayerRenderPlugin);
        app.configure_sets(
            Update,
            PlayerRenderPluginSet.run_if(in_state(LauncherStates::Playing)),
        );

        // --- Client and Server plugins below here ---

        // The server plugin will run only if we are the server (i.e. hosting)
        // and in the Playing state
        app.add_plugins(ServerPlugin);
        app.configure_sets(
            FixedUpdate,
            ServerPluginSet
                .run_if(is_server.and(in_state(LauncherStates::Playing)))
                .before(ClientPluginSet),
        );

        // The client plugin will run only in the Playing state
        app.add_plugins(ClientPlugin);
        app.configure_sets(
            FixedUpdate,
            ClientPluginSet.run_if(in_state(LauncherStates::Playing)),
        );

        // NOTE: For debugging purposes
        app.add_systems(
            OnEnter(LauncherStates::Playing),
            (setup_controller, create_a_single_test_chunk),
        );
        app.add_plugins(WASDCameraControllerPlugin);
        app.configure_sets(
            Update,
            WASDCameraControllerPluginSet.run_if(in_state(LauncherStates::Playing)),
        );
    }
}

fn setup_menu(mut commands: Commands, assets: Res<MainMenuAssets>) {
    debug!("Setting up main menu...");

    // Initialize the main menu resources from the loaded assets
    commands.insert_resource(MainMenuIcons {
        exit_icon: assets.exit_icon.clone(),
        right_icon: assets.right_icon.clone(),
        wrench_icon: assets.wrench_icon.clone(),
    });

    commands.spawn((
        Name::new("CameraMainMenuUI"),
        Camera2d,
        StateScoped(LauncherStates::MainMenu),
    ));

    // Spawn the main menu UI root node scoped to the MainMenu state.
    // This will trigger the MainMenuPlugin systems to populate it.
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
        StateScoped(LauncherStates::MainMenu),
    ));
}

fn handle_play_button_pressed(
    mut commands: Commands,
    mut ev_play: EventReader<ClientPlayClickEvent>,
    mut next_state: ResMut<NextState<LauncherStates>>,
    player_name: Res<PlayerNameSetting>,
) {
    for _ in ev_play.read() {
        // If the play button is pressed, transition to the Playing state
        // We spawn the ServerListener and ClientConnection entities here
        // to enable hosting mode.

        next_state.set(LauncherStates::Playing);

        let server = commands
            .spawn((
                Name::new("ServerListener"),
                ServerListener,
                StateScoped(LauncherStates::Playing),
            ))
            .id();

        commands.spawn((
            Name::new("ClientConnection"),
            HostConnection { server },
            ClientMetadata {
                username: (**player_name).clone(),
            },
            StateScoped(LauncherStates::Playing),
        ));
    }
}

fn handle_multiplayer_pressed(
    mut commands: Commands,
    mut ev_multiplayer: EventReader<ClientMultiplayerClickEvent>,
    mut next_state: ResMut<NextState<LauncherStates>>,
    player_name: Res<PlayerNameSetting>,
) {
    for event in ev_multiplayer.read() {
        // If the multiplayer button is pressed, transition to the Playing state
        // We spawn only the ClientConnection entity here to enable joining mode.

        next_state.set(LauncherStates::Playing);

        let addr = IpAddr::from_str(&event.address).unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));

        commands.spawn((
            Name::new("ClientConnection"),
            ClientConnection {
                address: SocketAddr::new(addr, SERVER_PORT),
            },
            ClientMetadata {
                username: (**player_name).clone(),
            },
            StateScoped(LauncherStates::Playing),
        ));
    }
}

fn setup_chat(mut commands: Commands) {
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
        StateScoped(LauncherStates::Playing),
    ));
}

fn setup_terrain_assets(mut commands: Commands) {
    debug!("Setting up terrain assets...");

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

fn setup_controller(mut commands: Commands) {
    commands.spawn((
        WASDCameraControllerBundle::default(),
        Camera3d::default(),
        Transform::from_xyz(60.0, 60.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("RTS Camera"),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(60.0, 60.0, 00.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Directional Light"),
    ));
}

fn create_a_single_test_chunk(mut ev_discover: EventWriter<TileDiscoverEvent>) {
    ev_discover.write(TileDiscoverEvent::new(Vec2::ZERO));
}
