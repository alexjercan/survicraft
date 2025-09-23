//! A Bevy plugin that serves as the main entry point for the game launcher.

use crate::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_simple_text_input::TextInputPlugin;
use iyes_progress::prelude::*;
use lightyear::prelude::{client::ClientPlugins, server::ServerPlugins};
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
    Connecting,
    Generating,
    Playing,
}

#[derive(Resource, Debug, Clone, Default)]
enum LauncherMode {
    Client(String), // Server address
    #[default]
    Host,
    Dedicated,
}

pub struct LauncherPlugin {
    pub render: bool,
}

impl LauncherPlugin {
    fn is_dedicated_server(&self, app: &App) -> bool {
        app.is_plugin_added::<DedicatedServerPlugin>()
    }
}

impl Plugin for LauncherPlugin {
    fn build(&self, app: &mut App) {
        let dedicated = self.is_dedicated_server(app);
        if dedicated && self.render {
            warn!("Dedicated server mode with rendering enabled. This is unusual.");
        }

        info!(
            "Launcher mode: {}",
            if dedicated {
                "Dedicated Server"
            } else {
                "Client/Host"
            }
        );

        if !dedicated {
            app.insert_resource(LauncherMode::default());
        } else {
            app.insert_resource(LauncherMode::Dedicated);
        }

        // Initialize the state machine
        app.init_state::<LauncherStates>();
        app.enable_state_scoped_entities::<LauncherStates>();

        // Multiplayer setup for lightyear. Here we set up both client and server plugins,
        // but we will control which ones are active using states and conditions. In case
        // the player is hosting, both client and server will be active. In case the player
        // is joining, only the client will be active.
        if !dedicated {
            app.add_plugins(ClientPlugins {
                tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
            });
        }
        app.add_plugins(ServerPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        });

        // Utility plugins. Text input plugin for handling text fields.
        if !dedicated {
            app.add_plugins(TextInputPlugin);
        }

        // Protocol plugin for handling message serialization and deserialization.
        app.add_plugins(ProtocolPlugin);

        // Progress tracking plugin.
        app.add_plugins(
            ProgressPlugin::<LauncherStates>::new()
                .with_state_transition(LauncherStates::Connecting, LauncherStates::Generating)
                .with_state_transition(LauncherStates::Generating, LauncherStates::Playing),
        );

        // Asset loading. This will transition from Loading to MainMenu state once done.
        app.add_systems(OnEnter(LauncherStates::Loading), setup_terrain_assets);
        app.add_systems(OnEnter(LauncherStates::Loading), setup_feature_assets);
        app.add_loading_state(
            LoadingState::new(LauncherStates::Loading)
                .continue_to_state(LauncherStates::MainMenu)
                // Load assets for the main menu
                .load_collection::<MainMenuAssets>(),
        );

        // Main Menu setup and event handling for starting the game.
        if !dedicated {
            app.add_systems(OnEnter(LauncherStates::MainMenu), setup_menu);
            app.add_plugins(MainMenuPlugin);
            app.add_systems(
                Update,
                (handle_play_button_pressed, handle_multiplayer_pressed)
                    .run_if(in_state(LauncherStates::MainMenu)),
            );
        } else {
            // If we are a dedicated server, skip the menu and go straight to connecting
            app.add_systems(
                OnEnter(LauncherStates::MainMenu),
                |mut next_state: ResMut<NextState<LauncherStates>>| {
                    next_state.set(LauncherStates::Connecting);
                },
            );
        }

        // Create connections when entering the Connecting state.
        if !dedicated {
            app.add_systems(OnEnter(LauncherStates::Connecting), setup_connecting_ui);
        }
        app.add_systems(OnEnter(LauncherStates::Connecting), setup_connections);
        app.add_systems(
            Update,
            check_connection_progress
                .track_progress::<LauncherStates>()
                .run_if(in_state(LauncherStates::Connecting)),
        );

        // Terrain generation setup and progress tracking.
        app.add_plugins(WorldGenerationPlugin {
            render: self.render,
        });
        if !dedicated {
            app.add_systems(OnEnter(LauncherStates::Generating), setup_loading_ui);
        }
        app.add_systems(
            OnEnter(LauncherStates::Generating),
            setup_terrain_generation,
        );
        app.add_systems(
            Update,
            check_terrain_generation_progress
                .track_progress::<LauncherStates>()
                .run_if(in_state(LauncherStates::Generating)),
        );

        // Physics setup. We disable interpolation and sleeping to ensure consistent physics
        app.add_plugins(
            PhysicsPlugins::default()
                .build()
                // disable Sync as it is handled by lightyear_avian
                .disable::<SyncPlugin>()
                // interpolation is handled by lightyear_frame_interpolation
                .disable::<PhysicsInterpolationPlugin>()
                // disable Sleeping plugin as it can mess up physics rollbacks
                .disable::<SleepingPlugin>(),
        );

        app.add_systems(OnEnter(LauncherStates::Playing), || {
            info!("Entered Playing state...");
        });

        if !dedicated {
            // Chat setup. We set up chat UI and related systems.
            app.add_systems(OnEnter(LauncherStates::Playing), setup_chat);
            app.add_plugins(ChatPlugin);
        }

        // The head camera controller will run only in the Playing state
        if !dedicated {
            app.add_systems(OnEnter(LauncherStates::Playing), setup_controller);
        }
        app.add_plugins(PlayerControllerPlugin {
            render: self.render,
        });

        if self.render {
            app.add_plugins(CommonRendererPlugin);
        }

        // --- Client and Server plugins below here ---

        if !dedicated {
            app.add_plugins(ClientPlugin);
        }
        app.add_plugins(ServerPlugin);
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
    mut ev_play: EventReader<ClientPlayClickEvent>,
    mut next_state: ResMut<NextState<LauncherStates>>,
    mut mode: ResMut<LauncherMode>,
) {
    for _ in ev_play.read() {
        // If the play button is pressed, transition to the Playing state
        // We spawn the ServerListener and ClientConnection entities here
        // to enable hosting mode.

        next_state.set(LauncherStates::Connecting);
        *mode = LauncherMode::Host;
    }
}

fn handle_multiplayer_pressed(
    mut ev_multiplayer: EventReader<ClientMultiplayerClickEvent>,
    mut next_state: ResMut<NextState<LauncherStates>>,
    mut mode: ResMut<LauncherMode>,
) {
    for event in ev_multiplayer.read() {
        // If the multiplayer button is pressed, transition to the Playing state
        // We spawn only the ClientConnection entity here to enable joining mode.

        next_state.set(LauncherStates::Connecting);
        *mode = LauncherMode::Client(event.address.clone());
    }
}

fn setup_connecting_ui() {}

fn setup_connections(mut commands: Commands, mode: Res<LauncherMode>) {
    info!("Setting up connections in mode: {:?}", *mode);

    match &*mode {
        LauncherMode::Host => {
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
                StateScoped(LauncherStates::Playing),
            ));
        }
        LauncherMode::Client(address) => {
            let addr = IpAddr::from_str(address).unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));

            commands.spawn((
                Name::new("ClientConnection"),
                ClientConnection {
                    address: SocketAddr::new(addr, SERVER_PORT),
                },
                StateScoped(LauncherStates::Playing),
            ));
        }
        LauncherMode::Dedicated => {
            commands.spawn((
                Name::new("ServerListener"),
                ServerListener,
                StateScoped(LauncherStates::Playing),
            ));
        }
    }
}

fn check_connection_progress(client_ready: Option<Res<ClientNetworkStateReady>>) -> Progress {
    match client_ready {
        None => {
            debug!("Connection progress: 1/1 (dedicated server)");

            Progress { done: 1, total: 1 }
        }
        Some(client_ready) => {
            let done = if **client_ready { 1 } else { 0 };
            debug!("Connection progress: {}/1", done);

            Progress { done, total: 1 }
        }
    }
}

fn setup_loading_ui() {
    // TODO: Implement a proper loading UI
}

fn setup_terrain_generation(mut commands: Commands) {
    commands.spawn((Name::new("InitializeTerrain"), InitializeTerrain));
}

fn check_terrain_generation_progress(terrain_progress: Res<TerrainGenerationProgress>) -> Progress {
    let total = terrain_progress.total_chunks.max(1); // Avoid division by zero
    debug!(
        "Terrain generation progress: {}/{} chunks",
        terrain_progress.generated_chunks, total
    );
    Progress {
        done: terrain_progress.generated_chunks,
        total: total,
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

fn setup_feature_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    debug!("Setting up feature assets...");

    // TODO: I want to load these from file, but for now, hardcode them
    // with some kind of cool syntax like:
    //
    // [feature]
    //     id="tree"
    //     name="Tree"
    //     [variant]
    //         id="sand"
    //         name="Palm Tree"
    //         threshold=0.9
    //         scene="gltf/decoration/nature/trees_A_cut.gltf#Scene0"
    //     [/variant]
    //     [variant]
    //         id="grass"
    //         name="Oak Tree"
    //         threshold=0.7
    //         scene="gltf/decoration/nature/trees_A_small.gltf#Scene0"
    //     [/variant]
    //     [variant]
    //         id="hills"
    //         name="Pine Tree"
    //         threshold=0.6
    //         scene="gltf/decoration/nature/trees_A_large.gltf#Scene0"
    //     [/variant]
    //     [variant]
    //         id="mountain"
    //         name="Fir Tree"
    //         threshold=0.8
    //         scene="gltf/decoration/nature/trees_A_large.gltf#Scene0"
    //     [/variant]
    // [/feature]
    commands.insert_resource(FeatureAssets::new(vec![FeatureAsset {
        id: "tree".to_string(),
        name: "Tree".to_string(),
        variants: vec![
            FeatureVariant {
                id: "sand".to_string(),
                name: "Palm Tree".to_string(),
                threshold: 0.9,
                scene: asset_server.load("gltf/decoration/nature/tree_single_A_cut.gltf#Scene0"),
            },
            FeatureVariant {
                id: "grass".to_string(),
                name: "Oak Tree".to_string(),
                threshold: 0.7,
                scene: asset_server.load("gltf/decoration/nature/tree_single_A.gltf#Scene0"),
            },
            FeatureVariant {
                id: "hills".to_string(),
                name: "Pine Tree".to_string(),
                threshold: 0.6,
                scene: asset_server.load("gltf/decoration/nature/tree_single_B.gltf#Scene0"),
            },
            FeatureVariant {
                id: "mountain".to_string(),
                name: "Fir Tree".to_string(),
                threshold: 0.8,
                scene: asset_server.load("gltf/decoration/nature/tree_single_B.gltf#Scene0"),
            },
        ],
    }]));
}

fn setup_controller(mut commands: Commands, mut ev_spawn: EventWriter<ClientSpawnPlayerEvent>) {
    ev_spawn.write(ClientSpawnPlayerEvent);

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(60.0, 60.0, 00.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Directional Light"),
        StateScoped(LauncherStates::Playing),
    ));
}
