use config::Config;
use registry::Registry;

pub mod address;
pub mod asn;
pub mod config;
pub mod key;
pub mod registry;
pub mod vici;

pub fn up(config: &Config, registry: &Registry) -> std::io::Result<()> {
    let public_key = key::private_key_to_public(config.private_key.as_bytes())?;
    dbg!(public_key);
    for local in &config.endpoints {
        let local_id = asn::encode_identity(
            &config.organization,
            &config.common_name,
            &local.serial_number,
        )
        .unwrap();
        dbg!(local_id);
        let local_addrs = address::expand_local_address(&local.address_family, &local.address);
        dbg!(local_addrs);
        for organization in registry {
            for node in &organization.nodes {
                if node.common_name == config.common_name {
                    continue;
                }
                for remote in &node.endpoints {
                    let remote_id = asn::encode_identity(
                        &organization.organization,
                        &node.common_name,
                        &remote.serial_number,
                    )
                    .unwrap();
                    dbg!(remote_id);
                    let remote_addrs =
                        address::expand_remote_address(&remote.address_family, &remote.address);
                    dbg!(remote_addrs);
                }
            }
        }
    }
    Ok(())
}
