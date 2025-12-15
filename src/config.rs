use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub organization: String,
    pub common_name: String,
    pub endpoints: Vec<Endpoint>,
    #[serde(default)]
    pub experimental: Experimental,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    pub serial_number: String,
    pub address_family: String,
    pub address: Option<String>,
    pub port: u16,

    pub updown: Option<String>,
    pub fwmark: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct Experimental {
    #[serde(default)]
    pub iptfs: bool,
}
