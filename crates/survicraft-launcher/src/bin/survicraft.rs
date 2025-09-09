use std::time::Duration;

use clap::Parser;
use lightyear::prelude::{client::ClientPlugins, server::ServerPlugins};
use survicraft_protocol::FIXED_TIMESTEP_HZ;

use survicraft_launcher::common::new_gui_app;

#[derive(clap::ValueEnum, Clone, Debug)]
enum Mode {
    Client,
    Host,
}

#[derive(Parser)]
#[command(name = "survicraft")]
#[command(version = "0.1")]
#[command(about = "Launcher for survicraft client", long_about = None)]
struct Cli {
    #[arg(value_enum)]
    mode: Mode,
}

fn main() {
    let cli = Cli::parse();

    match cli.mode {
        Mode::Client => {
            let mut app = new_gui_app();
            app.add_plugins(ClientPlugins {
                tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
            });
            app.add_plugins(survicraft_protocol::ProtocolPlugin);
            app.add_plugins(survicraft_client::ClientPlugin);
            app.run();
        }
        Mode::Host => {
            let mut app = new_gui_app();
            app.add_plugins(ClientPlugins {
                tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
            });
            app.add_plugins(ServerPlugins {
                tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
            });
            app.add_plugins(survicraft_protocol::ProtocolPlugin);
            app.add_plugins(survicraft_server::ServerPlugin);
            app.add_plugins(survicraft_client::ClientPlugin);
            app.run();
        }
    }
}
