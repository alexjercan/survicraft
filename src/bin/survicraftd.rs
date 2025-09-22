use clap::Parser;
use survicraft::prelude::*;

#[derive(Parser)]
#[command(name = "survicraftd")]
#[command(version = "0.1")]
#[command(about = "Dedicated server for survicraft client", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();

    let mut app = new_headless_app();
    app.add_plugins(LauncherPlugin { render: false });
    app.run();
}
