use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::serde_as;

#[derive(Deserialize, Serialize, Debug)]
pub struct Endpoint {
    pub send_port: u16,
    pub address_family: String,
    pub address: String,
}

#[serde_as]
#[derive(Deserialize, Serialize, Debug)]
pub struct Peer {
    #[serde_as(as = "Base64")]
    pub public_key: Vec<u8>,
    #[serde_as(as = "Base64")]
    pub operator_key: Vec<u8>,
    pub remarks: std::collections::HashMap<String, String>,
    pub endpoints: Vec<Endpoint>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PeerEnvelope {
    pub peer: Peer,
    pub signature: String,
}
