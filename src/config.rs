use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::serde_as;

#[serde_as]
#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub registry: String,
    #[serde_as(as = "Base64")]
    pub private_key: [u8; 32],
    pub vrf: String,
    pub transport: Vec<Transport>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Transport {
    pub address_family: String,
    pub send_port: u16,
    pub mtu: u32,
    pub ifprefix: String,
    pub ifgroup: u32,
    pub fwmark: u32,
    pub random_port: bool,
}
