use futures::stream::TryStreamExt;
use netlink_packet_core::{NetlinkMessage, NLM_F_REQUEST};
use netlink_packet_wireguard::{
    nlas::{WgAllowedIp, WgAllowedIpAttrs, WgDeviceAttrs, WgPeer, WgPeerAttrs},
    Wireguard, WireguardCmd,
};
use rand::Rng;
use ranet::wgctrl::*;
use rtnetlink::{
    packet::rtnl::constants,
    packet::rtnl::link::nlas::{Info, InfoKind, Nla},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ensure_link("wg0").await?;
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
