use futures::stream::TryStreamExt;
use netlink_packet_core::NLM_F_REQUEST;
use rand::Rng;
use rtnetlink::{
    packet::rtnl::constants,
    packet::rtnl::link::nlas::{Info, InfoKind, Nla},
};

pub struct LinkConfig {
    pub name: String,
    pub master: String,
    pub group: u32,
    pub mtu: u32,
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

pub async fn change_link_group(old: u32, new: u32) {
    let (rtc, rt, _) = rtnetlink::new_connection().unwrap();
    tokio::spawn(rtc);
    let mut resp = rt.link().get().execute();
    while let Ok(Some(link)) = resp.try_next().await {
        for nla in link.nlas.into_iter() {
            if let Nla::Group(group) = nla {
                if group == old {
                    let mut req = rt.link().set(link.header.index);
                    let msg = req.message_mut();
                    msg.nlas.push(Nla::Group(new));
                    req.execute().await.unwrap();
                }
            }
        }
    }
}

pub async fn remove_link_by_group(group: u32) {
    let (rtc, rt, _) = rtnetlink::new_connection().unwrap();
    tokio::spawn(rtc);
    let mut req = rt.link().del(0);
    let msg = req.message_mut();
    msg.nlas.push(Nla::Group(group));
    req.execute().await;
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

pub async fn ensure_link(cfg: &LinkConfig) -> Result<(), Box<dyn std::error::Error>> {
    let (rtc, rt, _) = rtnetlink::new_connection().unwrap();
    tokio::spawn(rtc);
    let mut req = if let Some(id) = link_id_by_name(&rt, &cfg.name).await {
        LinkRequest::Set(rt.link().set(id))
    } else {
        LinkRequest::Add(rt.link().add())
    };
    if let LinkRequest::Add(_) = req {
        req.message().nlas.push(Nla::IfName(cfg.name.clone()));
    }
    let msg = req.message();
    msg.header.flags = constants::IFF_UP;
    msg.nlas
        .push(Nla::Info(vec![Info::Kind(InfoKind::Wireguard)]));
    let master = link_id_by_name(&rt, &cfg.master).await.unwrap();
    msg.nlas.push(Nla::Master(master));
    msg.nlas.push(Nla::Group(cfg.group));
    msg.nlas.push(Nla::Mtu(cfg.mtu));
    req.execute().await?;
    let id = link_id_by_name(&rt, &cfg.name).await.unwrap();
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
        let mut rng = rand::thread_rng();
        let ip =
            std::net::Ipv6Addr::new(0xfe80, 0, 0, 0, rng.gen(), rng.gen(), rng.gen(), rng.gen());
        rt.address()
            .add(id, std::net::IpAddr::V6(ip), 64)
            .execute()
            .await?;
    }
    Ok(())
}
