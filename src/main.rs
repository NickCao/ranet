use argh::FromArgs;
use ranet::config::*;
use ranet::link::*;
use ranet::wgctrl::*;

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
    let cfg: Config = serde_json::from_slice(&std::fs::read(args.config).unwrap()).unwrap();
    let peers: Vec<ranet::Peer> =
        serde_json::from_slice(&std::fs::read(cfg.registry).unwrap()).unwrap();
    match args.command {
        Command::Up(_) => {
            for transport in cfg.transport {
                assert_ne!(transport.ifgroup, 0);
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
                        let lo = std::net::SocketAddr::new(
                            std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
                            0,
                        );
                        let addr = match hosts {
                            Ok(mut hosts) => hosts
                                .find(|addr| match endpoint.address_family.as_str() {
                                    "ip4" => addr.is_ipv4(),
                                    "ip6" => addr.is_ipv6(),
                                    _ => false,
                                })
                                .unwrap_or(lo),
                            Err(_) => lo,
                        };
                        let name = format!("{}{}", transport.ifprefix, endpoint.send_port);
                        ensure_link(&LinkConfig {
                            name: name.to_string(),
                            group: transport.ifgroup,
                            master: cfg.vrf.clone(),
                            mtu: transport.mtu,
                        })
                        .await?;
                        ensure_wireguard(&WireguardConfig {
                            name: name.to_string(),
                            private_key: cfg.private_key,
                            listen_port: endpoint.send_port,
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
            Ok(())
        }
        Command::Down(_) => {
            for transport in cfg.transport {
                assert_ne!(transport.ifgroup, 0);
                remove_link_by_group(transport.ifgroup).await;
            }
            Ok(())
        }
    }
}
