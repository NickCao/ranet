use config::Config;
use registry::Registry;

pub mod config;
pub mod registry;
pub mod vici;
pub mod asn;
pub mod key;

pub fn up(config: &Config, registry: &Registry) -> std::io::Result<()> {
    Ok(())
}
