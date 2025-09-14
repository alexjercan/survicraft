use bevy::prelude::*;
use clap::Parser;
use survicraft::prelude::*;

#[derive(Parser)]
#[command(name = "survicraft")]
#[command(version = "0.1")]
#[command(about = "Launcher for survicraft client", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();

    let mut app = new_gui_app();
    app.add_plugins(LauncherPlugin);
    app.run();
}
