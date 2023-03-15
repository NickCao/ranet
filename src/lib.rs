use config::Config;
use openssl::sha::sha256;
use registry::Registry;
use std::collections::HashSet;

pub mod address;
pub mod asn;
pub mod config;
pub mod key;
pub mod registry;
pub mod vici;

pub mod error {
    use std::{str::Utf8Error, string::FromUtf8Error};
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum Error {
        #[error("io error")]
        IO(#[from] std::io::Error),
        #[error("vici error")]
        Vici(#[from] rsvici::Error),
        #[error("semver error")]
        Semver(#[from] semver::Error),
        #[error("protocol error: {0:?}")]
        Protocol(Option<String>),
        #[error("from utf8 error")]
        FromUtf8(#[from] FromUtf8Error),
        #[error("utf8 error")]
        Utf8(#[from] Utf8Error),
        #[error("openssl error")]
        Openssl(#[from] openssl::error::ErrorStack),
        #[error("serde json error")]
        Json(#[from] serde_json::Error),
    }
}

pub async fn reconcile(
    socket: &str,
    config: &Config,
    registry: &Registry,
    key: &[u8],
) -> Result<(), error::Error> {
    let mut client = vici::Client::connect(socket).await?;

    client.load_key(key).await?;

    let public_key = key::private_key_to_public(key)?;
    let public_key = String::from_utf8(public_key)?;

    let mut desired = HashSet::<String>::default();

    for local in &config.endpoints {
        let local_id = asn::encode_identity(
            &config.organization,
            &config.common_name,
            &local.serial_number,
        )
        .unwrap();
        let local_addrs = address::local(&local.address_family, &local.address);
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
                    let remote_addrs = address::remote(&remote.address_family, &remote.address);
                    let name =
                        hex::encode(sha256(format!("{}-{}", &local_id, &remote_id).as_bytes()));
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
                        .await?;
                    client.initiate(&name).await?;
                }
            }
        }
    }

    let current = HashSet::<String>::from_iter(client.get_conns().await?.into_iter());

    for conn in current.difference(&desired) {
        client.unload_conn(conn).await?;
        client.terminate(conn).await?;
    }

    Ok(())
}
