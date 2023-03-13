use clap::{Parser, Subcommand};
use ranet::{config::Config, reconcile, registry::Registry};

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

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(&args.config)
        .unwrap();

    let config: Config = serde_json::from_reader(file).unwrap();

    match &args.command {
        Commands::Up => {
            let file = std::fs::OpenOptions::new()
                .read(true)
                .open(&args.registry)
                .unwrap();

            let registry: Registry = serde_json::from_reader(file).unwrap();

            reconcile(&config, &registry).await.unwrap();
        }
        Commands::Down => {
            reconcile(&config, &vec![]).await.unwrap();
        }
    }
}
