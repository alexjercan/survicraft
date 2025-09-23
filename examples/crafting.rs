use avian3d::prelude::*;
use bevy::prelude::*;
use clap::Parser;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};
use survicraft::prelude::*;

#[derive(Parser)]
#[command(name = "survicraft-crafting")]
#[command(version = "0.1")]
#[command(about = "Example for the survicraft crafting", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();

    let mut app = new_gui_app();

    app.add_plugins(PhysicsPlugins::default().build());

    app.add_systems(Startup, setup);
    app.add_plugins(PlayerControllerPlugin { render: true });

    app.run();
}

