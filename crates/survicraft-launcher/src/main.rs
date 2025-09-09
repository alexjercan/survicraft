use bevy::{
    diagnostic::DiagnosticsPlugin,
    log::{Level, LogPlugin},
    prelude::*,
    state::app::StatesPlugin,
    window::PresentMode,
};
use clap::Parser;

#[derive(clap::ValueEnum, Clone, Debug)]
enum Mode {
    Client,
    Server,
    Host,
}

#[derive(Parser)]
#[command(name = "survicraft")]
#[command(version = "0.1")]
#[command(about = "Launcher for survicraft client and server", long_about = None)]
struct Cli {
    #[arg(value_enum)]
    mode: Mode,
}

fn main() {
    let cli = Cli::parse();

    match cli.mode {
        Mode::Client => {
            let mut app = new_gui_app();
            app.add_plugins(survicraft_client::ClientPlugin);
            app.run();
        }
        Mode::Server => {
            let mut app = new_headless_app();
            app.add_plugins(survicraft_server::ServerPlugin);
            app.run();
        }
        Mode::Host => {
            let mut app = new_gui_app();
            app.add_plugins(survicraft_server::ServerPlugin);
            app.add_plugins(survicraft_client::ClientPlugin);
            app.run();
        }
    }
}

pub fn window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: format!("Survicraft - {}", env!("CARGO_PKG_VERSION")),
            resolution: (1024., 768.).into(),
            present_mode: PresentMode::AutoVsync,
            // set to true if we want to capture tab etc in wasm
            prevent_default_event_handling: true,
            ..Default::default()
        }),
        ..default()
    }
}

pub fn log_plugin() -> LogPlugin {
    LogPlugin {
        level: Level::INFO,
        filter: "wgpu=error,bevy_render=info,bevy_ecs=warn,bevy_time=warn,naga=warn".to_string(),
        ..default()
    }
}

fn new_gui_app() -> App {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build()
            .set(AssetPlugin {
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            })
            .set(log_plugin())
            .set(window_plugin()),
    );

    // TODO: Use a debug flag and plugin
    app.add_plugins(bevy_inspector_egui::bevy_egui::EguiPlugin::default());
    app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());

    app
}

pub fn new_headless_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        log_plugin(),
        StatesPlugin,
        DiagnosticsPlugin,
    ));
    app
}
