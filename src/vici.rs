use crate::{config, registry};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Result {
    pub success: bool,
    pub errmsg: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Conns {
    pub conns: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Version {
    pub daemon: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct Key {
    pub r#type: String,
    pub data: String,
}

#[derive(Debug, Serialize)]
pub struct Child {
    pub local_ts: Vec<String>,
    pub remote_ts: Vec<String>,
    pub updown: String,
    pub mode: String,
    pub dpd_action: String,
    pub set_mark_out: String,
    pub start_action: String,
}

impl Child {
    pub fn new(local: &config::Endpoint) -> Self {
        Self {
            local_ts: vec!["0.0.0.0/0".to_string(), "::/0".to_string()],
            remote_ts: vec!["0.0.0.0/0".to_string(), "::/0".to_string()],
            updown: local.updown.clone().unwrap_or_default(),
            mode: "tunnel".to_string(),
            dpd_action: "restart".to_string(),
            set_mark_out: local.fwmark.clone().unwrap_or_default(),
            start_action: "start".to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Auth {
    pub auth: String,
    pub pubkeys: Vec<String>,
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct Connection {
    pub version: u32,
    pub local_addrs: Vec<String>,
    pub remote_addrs: Vec<String>,
    pub local_port: u16,
    pub remote_port: u16,
    pub encap: bool,
    pub dpd_delay: u64,
    pub keyingtries: u32,
    pub unique: String,
    pub if_id_in: String,
    pub if_id_out: String,
    pub local: Auth,
    pub remote: Auth,
    pub children: HashMap<String, Child>,
}

impl Connection {
    pub fn new(
        local_addrs: Vec<String>,
        remote_addrs: Vec<String>,
        local_id: String,
        remote_id: String,
        local: &config::Endpoint,
        remote: &registry::Endpoint,
        local_pubkey: String,
        remote_pubkey: String,
    ) -> Self {
        Self {
            version: 2,
            local_addrs,
            remote_addrs,
            local_port: local.port,
            remote_port: remote.port,
            encap: true,
            dpd_delay: 60,
            keyingtries: 0,
            unique: "replace".to_string(),
            if_id_in: "%unique".to_string(),
            if_id_out: "%unique".to_string(),
            local: Auth {
                auth: "pubkey".to_string(),
                pubkeys: vec![local_pubkey],
                id: local_id,
            },
            remote: Auth {
                auth: "pubkey".to_string(),
                pubkeys: vec![remote_pubkey],
                id: remote_id,
            },
            children: HashMap::from([("default".to_string(), Child::new(local))]),
        }
    }
}
