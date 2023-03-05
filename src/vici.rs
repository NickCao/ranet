use std::collections::HashMap;

use crate::config;

#[derive(Debug, serde::Serialize)]
struct Child<'a> {
    local_ts: &'a [&'a str],
    remote_ts: &'a [&'a str],
    updown: Option<&'a str>,
    mode: &'a str,
    dpd_action: &'a str,
    set_mark_out: Option<&'a str>,
    start_action: &'a str,
}

#[derive(Debug, serde::Serialize)]
struct Auth<'a> {
    auth: &'a str,
    pubkeys: [&'a str; 1],
    id: &'a str,
}

#[derive(Debug, serde::Serialize)]
pub struct Connection<'a> {
    version: u32,
    local_addrs: &'a [&'a str],
    remote_addrs: &'a [&'a str],
    local_port: u16,
    remote_port: u16,
    encap: bool,
    dpd_delay: u64,
    keyingtries: u32,
    unique: &'a str,
    if_id_in: &'a str,
    if_id_out: &'a str,
    local: Auth<'a>,
    remote: Auth<'a>,
    children: HashMap<&'a str, Child<'a>>,
}

impl<'a> Connection<'a> {
    pub fn new<'b: 'a, 'c: 'a>(
        local: config::Transport<'b>,
        remote: config::Endpoint<'c>,
        local_pubkey: &'a str,
        remote_pubkey: &'a str,
    ) -> Self {
        Self {
            version: 2,
            local_addrs: &[],
            remote_addrs: &[],
            local_port: local.port,
            remote_port: remote.port,
            encap: true,
            dpd_delay: 60,
            keyingtries: 0,
            unique: "replace",
            if_id_in: "%unique",
            if_id_out: "%unique",
            local: Auth {
                auth: "pubkey",
                pubkeys: [local_pubkey],
                id: local.id,
            },
            remote: Auth {
                auth: "pubkey",
                pubkeys: [remote_pubkey],
                id: remote.id,
            },
            children: HashMap::from([(
                "default",
                Child {
                    local_ts: &["0.0.0.0/0", "::/0"],
                    remote_ts: &["0.0.0.0/0", "::/0"],
                    updown: local.updown,
                    mode: "tunnel",
                    dpd_action: "restart",
                    set_mark_out: local.fwmark,
                    start_action: "start",
                },
            )]),
        }
    }
}
