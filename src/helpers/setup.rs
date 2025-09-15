//! This module contains functions to create new Bevy apps with different configurations.

use bevy::{
    diagnostic::DiagnosticsPlugin,
    log::{Level, LogPlugin},
    prelude::*,
    state::app::StatesPlugin,
    window::PresentMode,
};

#[cfg(feature = "debug")]
use self::debug::DebugPlugin;

fn window_plugin() -> WindowPlugin {
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

fn log_plugin() -> LogPlugin {
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
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            })
            .set(log_plugin())
            .set(window_plugin()),
    );

    #[cfg(feature = "debug")]
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

#[cfg(feature = "debug")]
mod debug {
    use bevy::{prelude::*, render::view::RenderLayers};
    use bevy_inspector_egui::{
        DefaultInspectorConfigPlugin,
        bevy_egui::{EguiContext, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext},
        bevy_inspector, egui,
    };
    use iyes_perf_ui::prelude::*;

    #[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
    pub struct DebugPluginSet;

    pub struct DebugPlugin;

    impl Plugin for DebugPlugin {
        fn build(&self, app: &mut App) {
            app
                // we want Bevy to measure these values for us:
                .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
                .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
                .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
                .insert_resource(ShowAxes(true))
                .add_plugins(PerfUiPlugin)
                // Bevy egui inspector
                .add_plugins(EguiPlugin::default())
                .add_plugins(DefaultInspectorConfigPlugin) // adds default options and `InspectorEguiImpl`s
                .add_systems(EguiPrimaryContextPass, inspector_ui)
                // We need to order our system before PerfUiSet::Setup,
                // so that iyes_perf_ui can process any new Perf UI in the same
                // frame as we spawn the entities. Otherwise, Bevy UI will complain.
                .add_systems(
                    Update,
                    toggle
                        .before(iyes_perf_ui::PerfUiSet::Setup)
                        .in_set(DebugPluginSet),
                )
                .add_systems(Update, setup.in_set(DebugPluginSet).run_if(run_once));
        }
    }

    #[derive(Debug, Resource, Default, Clone, Deref, DerefMut)]
    struct ShowAxes(pub bool);

    fn setup(mut commands: Commands) {
        // create a simple Perf UI with default settings
        // and all entries provided by the crate:
        commands.spawn((Name::new("PerfUI"), PerfUiAllEntries::default()));

        commands.spawn((
            Camera2d,
            Camera {
                order: 1,
                ..default()
            },
            Name::new("Debug Camera"),
            RenderLayers::layer(1),
        ));
    }

    fn toggle(
        mut commands: Commands,
        q_root: Query<Entity, With<PerfUiRoot>>,
        kbd: Res<ButtonInput<KeyCode>>,
        mut show_axes: ResMut<ShowAxes>,
    ) {
        if kbd.just_pressed(KeyCode::F11) {
            if let Ok(e) = q_root.single() {
                // despawn the existing Perf UI
                commands.entity(e).despawn();
            } else {
                // create a simple Perf UI with default settings
                // and all entries provided by the crate:
                commands.spawn((Name::new("PerfUI"), PerfUiAllEntries::default()));
            }

            show_axes.0 = !show_axes.0;
        }
    }

    fn inspector_ui(world: &mut World) {
        let mut egui_context = world
            .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
            .single(world)
            .expect("EguiContext not found")
            .clone();

        egui::Window::new("UI").show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                // equivalent to `WorldInspectorPlugin`
                bevy_inspector::ui_for_world(world, ui);

                // // works with any `Reflect` value, including `Handle`s
                // let mut any_reflect_value: i32 = 5;
                // bevy_inspector::ui_for_value(&mut any_reflect_value, ui, world);

                // egui::CollapsingHeader::new("Materials").show(ui, |ui| {
                //     bevy_inspector::ui_for_assets::<StandardMaterial>(world, ui);
                // });

                // ui.heading("Entities");
                // bevy_inspector::ui_for_entities(world, ui);
            });
        });
    }
}
