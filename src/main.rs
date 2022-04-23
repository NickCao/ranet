use rand::Rng;
use ranet::link::*;
use ranet::wgctrl::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let private_key = rand::thread_rng().gen();
    let send_port = 12345;
    let group = 10;
    let master = 0;
    let mtu = 1400;
    let fwmark = 67;
    let peers: Vec<ranet::Peer> =
        reqwest::get("example.com")
            .await?
            .json()
            .await?;
    for peer in peers {
        for endpoint in peer.endpoints {
            let addr = match tokio::net::lookup_host((endpoint.address, send_port)).await {
                Ok(mut hosts) => hosts
                    .find(|addr| match endpoint.address_family.as_str() {
                        "ip4" => addr.is_ipv4(),
                        "ip6" => addr.is_ipv6(),
                        _ => true,
                    })
                    .unwrap_or(std::net::SocketAddr::new(
                        std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
                        0,
                    )),
                Err(_) => std::net::SocketAddr::new(
                    std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
                    0,
                ),
            };
            let name = format!("wg{}", endpoint.send_port);
            ensure_link(&LinkConfig {
                name: name.to_string(),
                group,
                master,
                mtu,
            })
            .await?;
            ensure_wireguard(&WireguardConfig {
                name: name.to_string(),
                private_key,
                listen_port: endpoint.send_port,
                fwmark,
                peer: PeerConfig {
                    public_key: peer.public_key,
                    endpoint: addr,
                    keep_alive: 20,
                },
            })
            .await?;
        }
    }
    Ok(())
}
