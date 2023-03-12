use config::Config;
use registry::Registry;

pub mod asn;
pub mod config;
pub mod key;
pub mod registry;
pub mod vici;

pub fn up(config: &Config, registry: &Registry) -> std::io::Result<()> {
    Ok(())
}
