use crate::error::Error;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use tracing::debug;

pub struct Client {
    client: rsvici::Client,
}

impl Client {
    pub async fn connect<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let client = rsvici::unix::connect(path).await?;
        Ok(Self { client })
    }
    pub async fn version(&mut self) -> Result<semver::Version, Error> {
        let v: Version = self.client.request("version", ()).await?;
        let v = semver::Version::parse(&v.version)?;
        Ok(v)
    }
    pub async fn load_key(&mut self, key: &[u8]) -> Result<(), Error> {
        let key = Key {
            r#type: "any",
            data: std::str::from_utf8(key)?,
        };
        let res: Status = self.client.request("load-key", key).await?;
        res.parse()
    }
    pub async fn load_conn(
        &mut self,
        name: &str,
        local: Endpoint,
        remote: Endpoint,
        updown: Option<String>,
        fwmark: Option<String>,
    ) -> Result<(), Error> {
        let conn = Connection::new(local, remote, updown, fwmark);
        let resp: Status = self
            .client
            .request("load-conn", HashMap::from([(name, conn)]))
            .await?;
        resp.parse()
    }
    pub async fn initiate(&mut self, name: &str) -> Result<(), Error> {
        let sas = self.list_sas(name).await?;

        for sa in sas.iter().flat_map(|v| v.values()) {
            if !sa.child_sas.is_empty() || sa.tasks_active.contains(&"CHILD_CREATE".to_string()) {
                return Ok(());
            }
        }

        debug!("initiating sa {}", name);

        let _res: Status = self
            .client
            .request(
                "initiate",
                Initiate {
                    ike: name,
                    child: "default",
                    timeout: -1,
                    init_limits: false,
                },
            )
            .await?;
        Ok(())
    }
    pub async fn terminate(&mut self, name: &str) -> Result<(), Error> {
        let _res: Status = self
            .client
            .request(
                "terminate",
                Terminate {
                    ike: name,
                    timeout: -1,
                },
            )
            .await?;
        Ok(())
    }
    pub async fn get_conns(&mut self) -> Result<Vec<String>, Error> {
        let res: Conns = self.client.request("get-conns", ()).await?;
        Ok(res.conns)
    }
    pub async fn unload_conn(&mut self, name: &str) -> Result<(), Error> {
        let res: Status = self.client.request("unload-conn", Unload { name }).await?;
        res.parse()
    }
    async fn list_sas(&mut self, name: &str) -> Result<Vec<HashMap<String, SA>>, Error> {
        let sas = self.client.stream_request::<ListSAs, SAs>(
            "list-sas",
            "list-sa",
            ListSAs { ike: name },
        );

        Ok(sas.try_collect::<Vec<_>>().await?)
    }
}

#[derive(Debug, Deserialize)]
struct Version {
    version: String,
}

#[derive(Debug, Serialize)]
struct Key<'a, 'b> {
    r#type: &'a str,
    data: &'b str,
}

#[derive(Debug, Deserialize)]
struct Status {
    success: bool,
    errmsg: Option<String>,
}

impl Status {
    fn parse(self) -> Result<(), Error> {
        match self {
            Status { success: true, .. } => Ok(()),
            Status {
                success: false,
                errmsg,
            } => Err(Error::Protocol(errmsg)),
        }
    }
}

#[derive(Debug, Deserialize)]
struct Conns {
    conns: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Unload<'a> {
    name: &'a str,
}

#[derive(Debug, Serialize)]
struct Initiate<'a, 'b> {
    ike: &'a str,
    child: &'b str,
    timeout: isize,
    init_limits: bool,
}

#[derive(Debug, Serialize)]
struct ListSAs<'a> {
    ike: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct SA {
    #[serde(default)]
    tasks_active: Vec<String>,
    child_sas: HashMap<String, serde::de::IgnoredAny>,
}

type SAs = HashMap<String, SA>;

#[derive(Debug, Serialize)]
struct Terminate<'a> {
    ike: &'a str,
    timeout: isize,
}

#[derive(Debug, Serialize)]
struct Child {
    // esp_proposals
    local_ts: Vec<String>,
    remote_ts: Vec<String>,
    updown: String,
    mode: &'static str,
    dpd_action: &'static str,
    set_mark_out: String,
    start_action: &'static str,
    close_action: &'static str,
}

#[derive(Debug, Serialize)]
struct Authentication {
    auth: &'static str,
    pubkeys: Vec<String>,
    id: String,
}

#[derive(Debug, Serialize)]
struct Connection {
    version: u32,
    local_addrs: Vec<String>,
    remote_addrs: Vec<String>,
    local_port: u16,
    remote_port: u16,
    // proposals
    // dscp
    encap: bool,
    mobike: bool,
    dpd_delay: u64,
    keyingtries: u32,
    unique: &'static str,
    if_id_in: &'static str,
    if_id_out: &'static str,
    local: Authentication,
    remote: Authentication,
    children: HashMap<&'static str, Child>,
}

pub struct Endpoint {
    pub id: String,
    pub addrs: Vec<String>,
    pub port: u16,
    pub pubkey: String,
}

impl Connection {
    fn new(
        local: Endpoint,
        remote: Endpoint,
        updown: Option<String>,
        fwmark: Option<String>,
    ) -> Self {
        Self {
            version: 2,
            local_addrs: local.addrs,
            remote_addrs: remote.addrs,
            local_port: local.port,
            remote_port: remote.port,
            encap: true,
            mobike: false,
            dpd_delay: 10,
            keyingtries: 0,
            unique: "replace",
            if_id_in: "%unique",
            if_id_out: "%unique",
            local: Authentication {
                auth: "pubkey",
                pubkeys: vec![local.pubkey],
                id: local.id,
            },
            remote: Authentication {
                auth: "pubkey",
                pubkeys: vec![remote.pubkey],
                id: remote.id,
            },
            children: HashMap::from([(
                "default",
                Child {
                    local_ts: vec!["0.0.0.0/0".to_string(), "::/0".to_string()],
                    remote_ts: vec!["0.0.0.0/0".to_string(), "::/0".to_string()],
                    updown: updown.unwrap_or_default(),
                    mode: "tunnel",
                    dpd_action: "restart",
                    set_mark_out: fwmark.unwrap_or_default(),
                    start_action: "none",
                    close_action: "none",
                },
            )]),
        }
    }
}
