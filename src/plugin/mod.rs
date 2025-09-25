//! A Bevy plugin that serves as the main entry point for the game launcher.

mod assets;
mod components;
mod controller;
mod network;
mod progress;
mod protocol;
mod render;
mod resources;
mod states;
mod ui;
mod world;
mod setup;

use avian3d::prelude::*;
use bevy::prelude::*;

pub use setup::{new_gui_app, new_headless_app};

pub struct LauncherPlugin {
    pub render: bool,
}

impl LauncherPlugin {
    fn is_dedicated_server(&self, app: &App) -> bool {
        app.is_plugin_added::<setup::DedicatedServerPlugin>()
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

        // Initialize the LauncherMode resource based on whether we are a dedicated server or not.
        if !dedicated {
            app.insert_resource(resources::LauncherMode::default());
        } else {
            app.insert_resource(resources::LauncherMode::Dedicated);
        }

        // Initialize the state machine
        app.init_state::<states::LauncherStates>();
        app.enable_state_scoped_entities::<states::LauncherStates>();

        app.add_plugins(network::NetworkPlugin { dedicated });
        app.add_plugins(progress::LauncherProgressPlugin);
        if !dedicated {
            app.add_plugins(ui::UIPlugin);
        }
        app.add_plugins(assets::AssetsPlugin);
        app.add_plugins(world::WorldGenerationPlugin {
            render: self.render,
        });
        app.add_plugins(
            PhysicsPlugins::default()
                .build()
                .disable::<SyncPlugin>()
                .disable::<PhysicsInterpolationPlugin>()
                .disable::<SleepingPlugin>(),
        );
        app.add_plugins(controller::PlayerControllerPlugin {
            render: self.render,
        });
        if self.render {
            app.add_plugins(render::CommonRendererPlugin);
        }

        // TODO: How can I refactor this part nicer?
        if dedicated {
            // If we are a dedicated server, skip the menu and go straight to connecting
            app.add_systems(
                OnEnter(states::LauncherStates::MainMenu),
                |mut next_state: ResMut<NextState<states::LauncherStates>>| {
                    next_state.set(states::LauncherStates::Connecting);
                },
            );
        }
        if !dedicated {
            // If we are not a dedicated server, spawn the player entity when entering Playing
            // state
            app.add_systems(
                OnEnter(states::LauncherStates::Playing),
                |mut ev_spawn: EventWriter<protocol::ClientSpawnPlayerEvent>| {
                    ev_spawn.write(protocol::ClientSpawnPlayerEvent);
                },
            );
        }
    }
}
