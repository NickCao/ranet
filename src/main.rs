use argh::FromArgs;
use ranet::config::*;
use ranet::link::*;
use ranet::wgctrl::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(FromArgs)]
/// ranet - redundant array of inexpensive tunnels
struct Args {
    /// path to config
    #[argh(option, short = 'c')]
    config: String,
    #[argh(subcommand)]
    command: Command,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Command {
    Up(Up),
    Down(Down),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "up")]
/// create or reconcile the tunnels
struct Up {}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "down")]
/// destroy the tunnels
struct Down {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = argh::from_env();
    let cfg: Config = serde_json::from_slice(&std::fs::read(args.config)?)?;
    let peers: Vec<ranet::Peer> = serde_json::from_slice(&std::fs::read(cfg.registry)?)?;
    // stale group and active group must be set
    assert_ne!(cfg.stale_group, 0);
    assert_ne!(cfg.active_group, 0);
    // address family must be one of ip4 or ip6
    assert!(cfg
        .transport
        .iter()
        .all(|t| ["ip4", "ip6"].contains(&t.address_family.as_str())));
    match args.command {
        Command::Up(_) => {
            let (conn, handle, _) = rtnetlink::new_connection()?;
            tokio::spawn(conn);
            // mark all existing interfaces as stale
            group_change(&handle, cfg.active_group, cfg.stale_group).await?;
            // get index of vrf device
            let master = index_query(&handle, &cfg.vrf).await?.unwrap();
            for transport in &cfg.transport {
                for peer in &peers {
                    for endpoint in &peer.endpoints {
                        if transport.address_family != endpoint.address_family {
                            continue;
                        }
                        let hosts = tokio::net::lookup_host((
                            endpoint.address.as_str(),
                            transport.send_port,
                        ))
                        .await;
                        let dummy = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);
                        let addr = match hosts {
                            Ok(mut hosts) => hosts
                                .find(|addr| match endpoint.address_family.as_str() {
                                    "ip4" => addr.is_ipv4(),
                                    "ip6" => addr.is_ipv6(),
                                    _ => false,
                                })
                                .unwrap_or(dummy),
                            Err(_) => dummy,
                        };
                        let name = format!("{}{}", transport.prefix, endpoint.send_port);
                        ensure(
                            &handle,
                            &LinkConfig {
                                name: name.to_string(),
                                group: cfg.active_group,
                                master,
                                mtu: transport.mtu,
                            },
                        )
                        .await?;
                        ensure_wireguard(&WireguardConfig {
                            name: name.to_string(),
                            private_key: cfg.private_key,
                            listen_port: if transport.random_port {
                                0
                            } else {
                                endpoint.send_port
                            },
                            fwmark: transport.fwmark,
                            peer: PeerConfig {
                                public_key: peer.public_key,
                                endpoint: addr,
                                keep_alive: 0,
                            },
                        })
                        .await?;
                    }
                }
            }
            // remove stale interfaces
            group_remove(&handle, cfg.stale_group).await?;
            Ok(())
        }
        Command::Down(_) => {
            let (conn, handle, _) = rtnetlink::new_connection()?;
            tokio::spawn(conn);
            // remove all interfaces
            group_remove(&handle, cfg.stale_group).await?;
            group_remove(&handle, cfg.active_group).await?;
            Ok(())
        }
    }
}
