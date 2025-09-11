use bevy::{
    diagnostic::DiagnosticsPlugin,
    log::{Level, LogPlugin},
    prelude::*,
    state::app::StatesPlugin,
    window::PresentMode,
};
use survicraft_common::debug::DebugPlugin;

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

pub fn new_gui_app() -> App {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build()
            .set(AssetPlugin {
                file_path: "../survicraft-assets/assets/".into(),
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            })
            .set(log_plugin())
            .set(window_plugin()),
    );

    app.add_plugins(DebugPlugin);

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
