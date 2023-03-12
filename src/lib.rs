use config::Config;
use registry::Registry;

pub mod config;
pub mod registry;
pub mod vici;
pub mod key;

pub fn up(config: &Config, registry: &Registry) -> std::io::Result<()> {
    Ok(())
}
