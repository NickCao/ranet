use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::serde_as;

#[serde_as]
#[derive(Deserialize, Serialize, Debug)]
/// ranet config
pub struct Config {
    /// path to registry file
    pub registry: String,
    #[serde_as(as = "Base64")]
    /// wireguard private key
    pub private_key: [u8; 32],
    /// vrf for interfaces
    pub vrf: String,
    /// wireguard config
    pub transport: Vec<Transport>,
    /// group for stale interfaces
    pub stale_group: u32,
    /// group for active interfaces
    pub active_group: u32,
    /// remarks for humans
    pub remarks: HashMap<String, String>,
}

#[derive(Deserialize, Serialize, Debug)]
/// wireguard config
pub struct Transport {
    /// address family, ip4 or ip6
    pub address_family: String,
    /// address
    pub address: String,
    /// wireguard send port
    pub send_port: u16,
    /// interface mtu
    pub mtu: u32,
    /// interface name prefix
    pub prefix: String,
    /// wireguard fwmark
    pub fwmark: u32,
    /// listen on random port
    pub random_port: bool,
}
