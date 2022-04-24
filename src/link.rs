use futures::stream::TryStreamExt;
use netlink_packet_core::ErrorMessage;
use rand::Rng;
use rtnetlink::{
    packet::rtnl::constants,
    packet::rtnl::link::nlas::{Info, InfoKind, Nla},
    Error::NetlinkError,
};

pub struct LinkConfig {
    pub name: String,
    pub master: u32,
    pub group: u32,
    pub mtu: u32,
}

const ERRNO_NO_SUCH_DEVICE: i32 = -19;

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

pub async fn group_change(
    handle: &rtnetlink::Handle,
    old: u32,
    new: u32,
) -> Result<(), rtnetlink::Error> {
    let mut resp = handle.link().get().execute();
    while let Some(link) = resp.try_next().await? {
        for nla in link.nlas.into_iter() {
            if let Nla::Group(group) = nla {
                if group == old {
                    let mut req = handle.link().set(link.header.index);
                    req.message_mut().nlas.push(Nla::Group(new));
                    req.execute().await?;
                }
            }
        }
    }
    Ok(())
}

pub async fn group_remove(handle: &rtnetlink::Handle, group: u32) -> Result<(), rtnetlink::Error> {
    let mut req = handle.link().del(0);
    req.message_mut().nlas.push(Nla::Group(group));
    let resp = req.execute().await;
    if let Err(NetlinkError(ErrorMessage {
        code: ERRNO_NO_SUCH_DEVICE,
        ..
    })) = resp
    {
        Ok(())
    } else {
        resp
    }
}

pub async fn index_query(
    handle: &rtnetlink::Handle,
    name: &str,
) -> Result<Option<u32>, rtnetlink::Error> {
    let mut links = handle.link().get().match_name(name.to_string()).execute();
    let resp = links.try_next().await;
    if let Err(NetlinkError(ErrorMessage {
        code: ERRNO_NO_SUCH_DEVICE,
        ..
    })) = resp
    {
        Ok(None)
    } else {
        Ok(resp?.map(|l| l.header.index))
    }
}

pub async fn ensure(
    handle: &rtnetlink::Handle,
    cfg: &LinkConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut req = if let Some(id) = index_query(handle, &cfg.name).await? {
        LinkRequest::Set(handle.link().set(id))
    } else {
        LinkRequest::Add(handle.link().add())
    };
    if let LinkRequest::Add(_) = req {
        req.message().nlas.push(Nla::IfName(cfg.name.clone()));
    }
    let msg = req.message();
    msg.header.flags = constants::IFF_UP;
    msg.nlas
        .push(Nla::Info(vec![Info::Kind(InfoKind::Wireguard)]));
    msg.nlas.push(Nla::Master(cfg.master));
    msg.nlas.push(Nla::Group(cfg.group));
    msg.nlas.push(Nla::Mtu(cfg.mtu));
    req.execute().await?;
    // FIXME: possible race condition
    let id = index_query(handle, &cfg.name).await?.unwrap();
    if !handle
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
        handle
            .address()
            .add(id, std::net::IpAddr::V6(ip), 64)
            .execute()
            .await?;
    }
    Ok(())
}
