use clap::Parser;

#[derive(Parser)]
#[command(name = "survicraft")]
#[command(version = "0.1")]
#[command(about = "Launcher for survicraft client", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
}
