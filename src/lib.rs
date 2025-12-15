use base64::prelude::{Engine, BASE64_URL_SAFE};
use config::Config;
use core::str;
use registry::Registry;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use tracing::{debug, debug_span, info, warn};

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
        #[error("pkcs8 error")]
        Openssl(#[from] ed25519_dalek::pkcs8::Error),
        #[error("serde json error")]
        Json(#[from] serde_json::Error),
    }
}

fn generate_name(existing: &HashSet<String>, data: &[u8]) -> String {
    let mut data = data.to_vec();
    loop {
        let name = BASE64_URL_SAFE.encode(&Sha256::digest(&data)[..3]);
        if existing.contains(&name) {
            data.push(0);
        } else {
            return name;
        }
    }
}

pub async fn reconcile(
    socket: &str,
    config: &Config,
    registry: &Registry,
    key: &[u8],
) -> Result<(), error::Error> {
    let _span_reconcile = debug_span!("reconcile").entered();

    let mut client = vici::Client::connect(socket).await?;

    debug!("connected to vici socket");

    client.load_key(key).await?;

    debug!("loaded private key");

    let public_key = key::private_key_to_public(str::from_utf8(key)?)?;

    debug!("derived public key");

    let mut desired = HashSet::<String>::default();

    for local in &config.endpoints {
        let _span_local = debug_span!("local").entered();

        let local_id = asn::encode_identity(
            &config.organization,
            &config.common_name,
            &local.serial_number,
        )
        .unwrap();

        debug!(
            "encoded local_id {} {} {}",
            config.organization, config.common_name, local.serial_number
        );

        let local_addrs = address::local(&local.address_family, &local.address);
        for organization in registry {
            let _span_organization = debug_span!("org", name = organization.organization).entered();

            for node in &organization.nodes {
                let _span_node = debug_span!("node", cn = node.common_name).entered();

                if node.common_name == config.common_name {
                    continue;
                }
                for remote in &node.endpoints {
                    let _span_endpoint =
                        debug_span!("endpoint", sn = remote.serial_number).entered();

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
                        generate_name(&desired, format!("{}-{}", &local_id, &remote_id).as_bytes());
                    desired.insert(name.clone());
                    let result = client
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
                            config.experimental.iptfs,
                        )
                        .await;

                    if let Err(err) = result {
                        warn!("load connection error: {}", err);
                        continue;
                    }

                    info!("loaded connection");

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
