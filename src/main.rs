use futures::{stream::TryStreamExt, StreamExt};
use genetlink::{GenetlinkError, GenetlinkHandle};
use netlink_packet_core::{NetlinkMessage, NLM_F_REQUEST};
use netlink_packet_generic::GenlMessage;
use netlink_packet_wireguard::{
    nlas::{WgDeviceAttrs, WgPeer, WgPeerAttrs},
    Wireguard, WireguardCmd,
};
use rand::Rng;
use rtnetlink::{
    packet::rtnl::constants,
    packet::{
        rtnl::link::nlas::{Info, InfoKind, Nla},
        LinkMessage,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ensure_link("wg0").await?;
    Ok(())
}

enum LinkRequest {
    Add(rtnetlink::LinkAddRequest),
    Set(rtnetlink::LinkSetRequest),
}

impl LinkRequest {
    fn message(&mut self) -> &mut rtnetlink::packet::LinkMessage {
        match self {
            Self::Add(r) => r.message_mut(),
            Self::Set(r) => r.message_mut(),
        }
    }
    async fn execute(self) -> Result<(), rtnetlink::Error> {
        match self {
            Self::Add(r) => r.execute().await,
            Self::Set(r) => r.execute().await,
        }
    }
}

async fn link_id_by_name(handle: &rtnetlink::Handle, name: &str) -> Option<u32> {
    let mut links = handle.link().get().match_name(name.to_string()).execute();
    // FIXME: check error
    if let Ok(Some(link)) = links.try_next().await {
        Some(link.header.index)
    } else {
        None
    }
}

async fn ensure_link(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (rtc, rt, _) = rtnetlink::new_connection().unwrap();
    tokio::spawn(rtc);
    let mut req = if let Some(id) = link_id_by_name(&rt, name).await {
        LinkRequest::Set(rt.link().set(id))
    } else {
        LinkRequest::Add(rt.link().add())
    };
    if let LinkRequest::Add(_) = req {
        req.message().nlas.push(Nla::IfName(name.to_string()));
    }
    let msg = req.message();
    msg.header.flags = constants::IFF_UP;
    msg.nlas
        .push(Nla::Info(vec![Info::Kind(InfoKind::Wireguard)]));
    msg.nlas.push(Nla::Master(136));
    msg.nlas.push(Nla::Group(1));
    msg.nlas.push(Nla::Mtu(1400));
    req.execute().await?;
    let id = link_id_by_name(&rt, name).await.unwrap();
    if !rt
        .address()
        .get()
        .set_link_index_filter(id)
        .execute()
        .try_collect::<Vec<rtnetlink::packet::rtnl::AddressMessage>>()
        .await
        .into_iter()
        .flatten()
        .any(|addr| addr.header.scope == rtnetlink::packet::rtnl::constants::RT_SCOPE_LINK)
    {
        // ll not found, generate a random one
    }
    Ok(())
}

/*
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
*/
