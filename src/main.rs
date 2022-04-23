use futures::stream::TryStreamExt;
use futures::StreamExt;
use genetlink::{GenetlinkError, GenetlinkHandle};
use netlink_packet_core::{NetlinkMessage, NetlinkPayload, NLM_F_DUMP, NLM_F_REQUEST};
use netlink_packet_generic::GenlMessage;
use netlink_packet_wireguard::{
    nlas::{WgAllowedIpAttrs, WgDeviceAttrs, WgPeer, WgPeerAttrs},
    Wireguard, WireguardCmd,
};
use rand::Rng;
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
        cmd: WireguardCmd::SetDevice,
        nlas: vec![
            WgDeviceAttrs::IfName(IFACE_NAME.to_string()),
            WgDeviceAttrs::PrivateKey(rand::thread_rng().gen::<[u8; 32]>()),
            WgDeviceAttrs::ListenPort(9999),
            WgDeviceAttrs::Fwmark(55),
            WgDeviceAttrs::Flags(netlink_packet_wireguard::constants::WGDEVICE_F_REPLACE_PEERS),
            WgDeviceAttrs::Peers(vec![WgPeer(vec![
                WgPeerAttrs::PublicKey(rand::thread_rng().gen::<[u8; 32]>()),
                WgPeerAttrs::PersistentKeepalive(25),
            ])]),
        ],
    });
    let mut nlmsg = NetlinkMessage::from(genlmsg);
    nlmsg.header.flags = NLM_F_REQUEST;
    handle.notify(nlmsg).await?;
    Ok(handle)
}

