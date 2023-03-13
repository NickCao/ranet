use config::Config;
use registry::Registry;
use sha2::{Digest, Sha256};
use std::collections::HashSet;

pub mod address;
pub mod asn;
pub mod config;
pub mod key;
pub mod registry;
pub mod vici;

pub async fn reconcile(config: &Config, registry: &Registry) -> std::io::Result<()> {
    let mut client = vici::Client::connect("/run/charon.vici").await.unwrap();

    client.load_key(&config.private_key).await.unwrap();

    let public_key = key::private_key_to_public(config.private_key.as_bytes())?;
    let public_key = String::from_utf8(public_key).unwrap();

    let mut desired = HashSet::<String>::default();

    for local in &config.endpoints {
        let local_id = asn::encode_identity(
            &config.organization,
            &config.common_name,
            &local.serial_number,
        )
        .unwrap();
        let local_addrs = address::expand_local_address(&local.address_family, &local.address);
        for organization in registry {
            for node in &organization.nodes {
                if node.common_name == config.common_name {
                    continue;
                }
                for remote in &node.endpoints {
                    if remote.address_family != local.address_family {
                        continue;
                    }
                    let remote_id = asn::encode_identity(
                        &organization.organization,
                        &node.common_name,
                        &remote.serial_number,
                    )
                    .unwrap();
                    let remote_addrs =
                        address::expand_remote_address(&remote.address_family, &remote.address);
                    let name = hex::encode(Sha256::digest(format!("{}-{}", &local_id, &remote_id)));
                    desired.insert(name.clone());
                    client
                        .load_conn(
                            &name,
                            vici::Endpoint {
                                id: local_id.clone(),
                                addrs: local_addrs.clone(),
                                port: local.port,
                                pubkey: public_key.clone(),
                            },
                            vici::Endpoint {
                                id: remote_id,
                                addrs: remote_addrs,
                                port: remote.port,
                                pubkey: organization.public_key.clone(),
                            },
                            local.updown.clone(),
                            local.fwmark.clone(),
                        )
                        .await
                        .unwrap();
                }
            }
        }
    }

    let current = HashSet::<String>::from_iter(client.get_conns().await.unwrap().into_iter());

    for conn in current.difference(&desired) {
        client.unload_conn(conn).await.unwrap();
    }

    Ok(())
}
