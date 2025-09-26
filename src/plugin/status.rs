use bevy::prelude::*;
use lightyear::prelude::*;

use super::states::*;
use crate::prelude::*;

pub(super) struct StatusPlugin;

impl Plugin for StatusPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StatusBarPlugin);
        if !app.is_plugin_added::<bevy::diagnostic::FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
        }

        app.add_systems(OnEnter(LauncherStates::Playing), setup_status);
        app.add_systems(
            Update,
            update_fps_status.run_if(in_state(LauncherStates::Playing)),
        );
        app.add_systems(
            Update,
            update_latency_status.run_if(in_state(LauncherStates::Playing)),
        );
    }
}

#[derive(Component)]
struct StatusBarItemFPS;

#[derive(Component)]
struct StatusBarItemLatency;

fn setup_status(
    mut commands: Commands,
    show_fps: Res<DisplayFPSSetting>,
    show_latency: Res<DisplayLatencySetting>,
    asset_server: Res<AssetServer>,
) {
    if **show_fps {
        commands.spawn((
            Name::new("StatusBarItem FPS"),
            StatusBarItemFPS,
            StatusBarItem {
                icon: Some(asset_server.load("textures/GameIcons/fps.png")),
                value: 0,
                label: "fps".to_string(),
                mapping: vec![
                    (Some(30), Color::srgb(1.0, 0.0, 0.0)),
                    (Some(60), Color::srgb(1.0, 1.0, 0.0)),
                    (None, Color::srgb(0.0, 1.0, 0.0)),
                ],
            },
            StateScoped(LauncherStates::Playing),
        ));
    }

    if **show_latency {
        commands.spawn((
            Name::new("StatusBarItem Latency"),
            StatusBarItemLatency,
            StatusBarItem {
                icon: Some(asset_server.load("textures/GameIcons/latency.png")),
                value: 0,
                label: "ms".to_string(),
                mapping: vec![
                    (Some(100), Color::srgb(0.0, 1.0, 0.0)),
                    (Some(200), Color::srgb(1.0, 1.0, 0.0)),
                    (None, Color::srgb(1.0, 0.0, 0.0)),
                ],
            },
            StateScoped(LauncherStates::Playing),
        ));
    }
}

fn update_fps_status(
    store: Res<bevy::diagnostic::DiagnosticsStore>,
    mut q_status: Query<&mut StatusBarItem, With<StatusBarItemFPS>>,
) {
    let fps = store
        .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.average())
        .map(|v| v.round() as u32)
        .unwrap_or(0);

    for mut status in &mut q_status {
        status.value = fps;
    }
}

fn update_latency_status(
    q_link: Query<&Link, With<Client>>,
    mut q_status: Query<&mut StatusBarItem, With<StatusBarItemLatency>>,
) {
    let latency = q_link
        .iter()
        .map(|link| link.stats.rtt.as_millis() as u32)
        .max()
        .unwrap_or(0);

    for mut status in &mut q_status {
        status.value = latency;
    }
}
