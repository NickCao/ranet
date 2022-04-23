use futures::stream::TryStreamExt;
use futures::StreamExt;
use genetlink::{GenetlinkError, GenetlinkHandle};
use ipnetwork::IpNetwork;
use netlink_packet_core::{NetlinkMessage, NetlinkPayload, NLM_F_DUMP, NLM_F_REQUEST};
use netlink_packet_generic::GenlMessage;
use netlink_packet_wireguard::{
    nlas::{WgAllowedIpAttrs, WgDeviceAttrs, WgPeerAttrs},
    Wireguard, WireguardCmd,
};
use rtnetlink::{
    packet::rtnl::constants,
    packet::rtnl::link::nlas::{Info, InfoKind, Nla},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handle = _create_wg().await;
    assert!(handle.is_ok());
    let handle = _print_wg().await;
    assert!(handle.is_ok());
    Ok(())
}

const IFACE_NAME: &str = "wg0";

async fn _create_wg() -> Result<rtnetlink::LinkHandle, rtnetlink::Error> {
    let (conn, handle, _) = rtnetlink::new_connection().unwrap();
    tokio::spawn(conn);
    let mut link_handle = handle.link();
    let mut req = link_handle.add();
    let mutator = req.message_mut();
    let info = Nla::Info(vec![Info::Kind(InfoKind::Wireguard)]);
    mutator.header.flags = constants::IFF_UP;
    mutator.nlas.push(info);
    mutator.nlas.push(Nla::Master(136));
    mutator.nlas.push(Nla::Group(1));
    mutator.nlas.push(Nla::Mtu(1400));
    mutator.nlas.push(Nla::IfName(IFACE_NAME.to_owned()));
    req.execute().await?;
    let mut query = link_handle
        .get()
        .match_name(IFACE_NAME.to_owned())
        .execute();
    while let Ok(Some(link)) = query.try_next().await {
        let ip: ipnetwork::IpNetwork = "fe80::1/64".parse().unwrap();
        handle
            .address()
            .add(link.header.index, ip.ip(), ip.prefix())
            .execute()
            .await?
    }

    Ok(link_handle)
}

async fn _print_wg() -> Result<GenetlinkHandle, GenetlinkError> {
    let (conn, mut handle, _) = genetlink::new_connection().unwrap();
    tokio::spawn(conn);
    let genlmsg: GenlMessage<Wireguard> = GenlMessage::from_payload(Wireguard {
        cmd: WireguardCmd::GetDevice,
        nlas: vec![WgDeviceAttrs::IfName("wg0".to_string())],
    });
    let mut nlmsg = NetlinkMessage::from(genlmsg);
    nlmsg.header.flags = NLM_F_REQUEST | NLM_F_DUMP;
    let mut resp = handle.request(nlmsg).await?;
    while let Some(result) = resp.next().await {
        let rx_packet = result.unwrap();
        match rx_packet.payload {
            NetlinkPayload::InnerMessage(genlmsg) => {
                print_wg_payload(genlmsg.payload);
            }
            NetlinkPayload::Error(e) => {
                eprintln!("Error: {:?}", e.to_io());
            }
            _ => (),
        };
    }
    Ok(handle)
}

fn print_wg_payload(wg: Wireguard) {
    for nla in &wg.nlas {
        match nla {
            WgDeviceAttrs::IfIndex(v) => println!("IfIndex: {}", v),
            WgDeviceAttrs::IfName(v) => println!("IfName: {}", v),
            WgDeviceAttrs::PrivateKey(_) => println!("PrivateKey: (hidden)"),
            WgDeviceAttrs::PublicKey(v) => println!("PublicKey: {:?}", v),
            WgDeviceAttrs::ListenPort(v) => println!("ListenPort: {}", v),
            WgDeviceAttrs::Fwmark(v) => println!("Fwmark: {}", v),
            WgDeviceAttrs::Peers(nlas) => {
                for peer in nlas {
                    println!("Peer: ");
                    print_wg_peer(peer);
                }
            }
            _ => (),
        }
    }
}

fn print_wg_peer(nlas: &[WgPeerAttrs]) {
    for nla in nlas {
        match nla {
            WgPeerAttrs::PublicKey(v) => println!("  PublicKey: {:?}", v),
            WgPeerAttrs::PresharedKey(_) => println!("  PresharedKey: (hidden)"),
            WgPeerAttrs::Endpoint(v) => println!("  Endpoint: {}", v),
            WgPeerAttrs::PersistentKeepalive(v) => println!("  PersistentKeepalive: {}", v),
            WgPeerAttrs::LastHandshake(v) => println!("  LastHandshake: {:?}", v),
            WgPeerAttrs::RxBytes(v) => println!("  RxBytes: {}", v),
            WgPeerAttrs::TxBytes(v) => println!("  TxBytes: {}", v),
            WgPeerAttrs::AllowedIps(nlas) => {
                for ip in nlas {
                    print_wg_allowedip(ip);
                }
            }
            _ => (),
        }
    }
}

fn print_wg_allowedip(nlas: &[WgAllowedIpAttrs]) -> Option<()> {
    let ipaddr = nlas.iter().find_map(|nla| {
        if let WgAllowedIpAttrs::IpAddr(addr) = nla {
            Some(*addr)
        } else {
            None
        }
    })?;
    let cidr = nlas.iter().find_map(|nla| {
        if let WgAllowedIpAttrs::Cidr(cidr) = nla {
            Some(*cidr)
        } else {
            None
        }
    })?;
    println!("  AllowedIp: {}/{}", ipaddr, cidr);
    Some(())
}
