use config::Config;
use registry::Registry;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub mod address;
pub mod asn;
pub mod config;
pub mod key;
pub mod registry;
pub mod vici;

pub async fn up(config: &Config, registry: &Registry) -> std::io::Result<()> {
    let mut client = vici::Client::connect("/run/charon.vici").await.unwrap();

    let ver = client.version().await.unwrap();

    let req = semver::VersionReq::parse(">=5.9.6").unwrap();
    assert!(req.matches(&ver));

    dbg!(ver);

    client.load_key(&config.private_key).await.unwrap();

    /*
    let public_key = key::private_key_to_public(config.private_key.as_bytes())?;
    let public_key = String::from_utf8(public_key).unwrap();
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
                    let conn = vici::Connection::new(
                        local_addrs.clone(),
                        remote_addrs.clone(),
                        local_id.clone(),
                        remote_id.clone(),
                        &local,
                        &remote,
                        public_key.clone(),
                        organization.public_key.clone(),
                    );
                    let name = hex::encode(Sha256::digest(format!("{}-{}", &local_id, &remote_id)));
                    let resp: vici::Status = session
                        .request(
                            "load-conn",
                            HashMap::<String, vici::Connection>::from([(name, conn)]),
                        )
                        .await
                        .unwrap();
                }
            }
        }
    }
    */
    Ok(())
}
