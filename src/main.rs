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
    /// path to private key
    #[arg(short, long)]
    key: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Up,
    Down,
}

#[tokio::main]
async fn main() -> Result<(), ranet::error::Error> {
    let args = Args::parse();

    let config = tokio::fs::read(&args.config).await?;
    let config: Config = serde_json::from_slice(&config)?;

    let key = tokio::fs::read(&args.key).await?;

    match &args.command {
        Commands::Up => {
            let registry = tokio::fs::read(&args.registry).await?;
            let registry: Registry = serde_json::from_slice(&registry)?;

            reconcile(&config, &registry, &key).await?;
        }
        Commands::Down => {
            reconcile(&config, &vec![], &key).await?;
        }
    }

    Ok(())
}
