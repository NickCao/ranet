use clap::{Parser, Subcommand};
use ranet::{config::Config, registry::Registry, up};

/// ranet
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// path to config file
    #[arg(short, long)]
    config: String,
    /// path to registry file
    #[arg(short, long)]
    registry: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Up,
    Down,
}

fn main() {
    let args = Args::parse();

    match &args.command {
        Commands::Up => {
            let cfgfile = std::fs::OpenOptions::new().read(true).open(&args.config).unwrap();
            let regfile = std::fs::OpenOptions::new().read(true).open(&args.registry).unwrap();
            let config: Config = serde_json::from_reader(cfgfile).unwrap();
            let registry: Registry = serde_json::from_reader(regfile).unwrap();
            up(&config, &registry).unwrap();
        }
        Commands::Down => {}
    }
}
