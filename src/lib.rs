use config::Config;
use registry::Registry;

pub mod asn;
pub mod config;
pub mod key;
pub mod registry;
pub mod vici;

pub fn up(config: &Config, registry: &Registry) -> std::io::Result<()> {
    let public_key = key::private_key_to_public(config.private_key.as_bytes())?;
    dbg!(public_key);
    for endpoint in &config.endpoints {
        let identity = asn::encode_identity(
            &config.organization,
            &config.common_name,
            &endpoint.serial_number,
        )
        .unwrap();
        dbg!(identity);
    }
    Ok(())
}
