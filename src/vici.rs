use crate::config;
use std::{collections::HashMap, vec};
/*

#[derive(Debug, serde::Serialize)]
struct Child {
    local_ts: Vec<String>,
    remote_ts: Vec<String>,
    updown: Option<String>,
    mode: String,
    dpd_action: String,
    set_mark_out: Option<String>,
    start_action: String,
}

impl Child {
    pub fn new(local: &config::Endpoint) -> Self {
        Self {
            local_ts: vec!["0.0.0.0/0".to_string(), "::/0".to_string()],
            remote_ts: vec!["0.0.0.0/0".to_string(), "::/0".to_string()],
            updown: local.updown.clone(),
            mode: "tunnel".to_string(),
            dpd_action: "restart".to_string(),
            set_mark_out: local.fwmark.clone(),
            start_action: "start".to_string(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct Auth {
    auth: String,
    pubkeys: Vec<String>,
    id: String,
}

#[derive(Debug, serde::Serialize)]
pub struct Connection {
    version: u32,
    local_addrs: Vec<String>,
    remote_addrs: Vec<String>,
    local_port: u16,
    remote_port: u16,
    encap: bool,
    dpd_delay: u64,
    keyingtries: u32,
    unique: String,
    if_id_in: String,
    if_id_out: String,
    local: Auth,
    remote: Auth,
    children: HashMap<String, Child>,
}

impl Connection {
    pub fn new(
        &config: &config::Config,
        local: &config::Endpoint,
        remote: &config::Endpoint,
        local_pubkey: &str,
        remote_pubkey: &str,
    ) -> Self {
        Self {
            version: 2,
            local_addrs: vec![],
            remote_addrs: vec![],
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
                pubkeys: vec![local_pubkey.to_string()],
                id: local.id.clone(),
            },
            remote: Auth {
                auth: "pubkey".to_string(),
                pubkeys: vec![remote_pubkey.to_string()],
                id: remote.id.clone(),
            },
            children: HashMap::from([("default".to_string(), Child::new(local))]),
        }
    }
}
*/
