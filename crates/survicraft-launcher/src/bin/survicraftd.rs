use bevy::prelude::*;
use std::time::Duration;
use survicraft_server::ServerListener;

use clap::Parser;
use lightyear::prelude::server::ServerPlugins;
use survicraft_common::setup::new_headless_app;
use survicraft_protocol::FIXED_TIMESTEP_HZ;

#[derive(Parser)]
#[command(name = "survicraftd")]
#[command(version = "0.1")]
#[command(about = "Server for the survicraft game.", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();

    let mut app = new_headless_app();
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
    });
    app.add_plugins(survicraft_protocol::ProtocolPlugin);
    app.add_plugins(survicraft_server::ServerPlugin);

    app.add_systems(Startup, startup);

    app.run();
}

fn startup(mut commands: Commands) {
    commands.spawn((Name::new("ServerListener"), ServerListener));
}
