use netlink_packet_core::{NetlinkMessage, NLM_F_REQUEST};
use netlink_packet_generic::GenlMessage;
use netlink_packet_wireguard::{
    nlas::{WgAllowedIp, WgAllowedIpAttrs, WgDeviceAttrs, WgPeer, WgPeerAttrs},
    Wireguard, WireguardCmd,
};
use rtnetlink::packet::{AF_INET, AF_INET6};

pub struct WireguardConfig {
    pub name: String,
    pub private_key: [u8; 32],
    pub listen_port: u16,
    pub fwmark: u32,
    pub peer: PeerConfig,
}

pub struct PeerConfig {
    pub public_key: [u8; 32],
    pub keep_alive: u16,
    pub endpoint: std::net::SocketAddr,
}

pub async fn ensure_wireguard(cfg: &WireguardConfig) -> Result<(), Box<dyn std::error::Error>> {
    let (gec, mut ge, _) = genetlink::new_connection()?;
    tokio::spawn(gec);
    let mut msg = NetlinkMessage::from(GenlMessage::from_payload(Wireguard {
        cmd: WireguardCmd::SetDevice,
        nlas: vec![
            WgDeviceAttrs::Flags(netlink_packet_wireguard::constants::WGDEVICE_F_REPLACE_PEERS),
            WgDeviceAttrs::IfName(cfg.name.clone()),
            WgDeviceAttrs::PrivateKey(cfg.private_key),
            WgDeviceAttrs::ListenPort(cfg.listen_port),
            WgDeviceAttrs::Fwmark(cfg.fwmark),
            WgDeviceAttrs::Peers(vec![WgPeer(vec![
                WgPeerAttrs::Flags(
                    netlink_packet_wireguard::constants::WGPEER_F_REPLACE_ALLOWEDIPS,
                ),
                WgPeerAttrs::PublicKey(cfg.peer.public_key),
                WgPeerAttrs::PersistentKeepalive(cfg.peer.keep_alive),
                WgPeerAttrs::Endpoint(cfg.peer.endpoint),
                WgPeerAttrs::AllowedIps(vec![
                    WgAllowedIp(vec![
                        WgAllowedIpAttrs::Family(AF_INET),
                        WgAllowedIpAttrs::IpAddr(std::net::IpAddr::V4(std::net::Ipv4Addr::new(
                            0, 0, 0, 0,
                        ))),
                        WgAllowedIpAttrs::Cidr(0),
                    ]),
                    WgAllowedIp(vec![
                        WgAllowedIpAttrs::Family(AF_INET6),
                        WgAllowedIpAttrs::IpAddr(std::net::IpAddr::V6(std::net::Ipv6Addr::new(
                            0, 0, 0, 0, 0, 0, 0, 0,
                        ))),
                        WgAllowedIpAttrs::Cidr(0),
                    ]),
                ]),
            ])]),
        ],
    }));
    msg.header.flags = NLM_F_REQUEST;
    ge.notify(msg).await?;
    Ok(())
}
