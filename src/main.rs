use rand::Rng;
use ranet::link::*;
use ranet::wgctrl::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ensure_link(&LinkConfig {
        name: "wg0".to_string(),
        group: 1,
        master: 0,
        mtu: 1400,
    })
    .await?;
    ensure_wireguard(&WireguardConfig {
        name: "wg0".to_string(),
        private_key: rand::thread_rng().gen(),
        listen_port: 12345,
        fwmark: 67,
        peer: PeerConfig {
            public_key: rand::thread_rng().gen(),
            endpoint: std::net::SocketAddr::new(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(1, 2, 3, 4)),
                5678,
            ),
            keep_alive: 15,
        },
    })
    .await?;
    Ok(())
}
