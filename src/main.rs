use rtnetlink::{
    new_connection,
    packet::rtnl::link::nlas::{Info, InfoKind, Nla},
    Error, LinkHandle,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handle = _create_wg().await;
    assert!(handle.is_ok());
    Ok(())
}

const IFACE_NAME: &str = "wg0";

async fn _create_wg() -> Result<LinkHandle, Error> {
    let (conn, handle, _) = new_connection().unwrap();
    tokio::spawn(conn);
    let link_handle = handle.link();
    let mut req = link_handle.add();
    let mutator = req.message_mut();
    let info = Nla::Info(vec![Info::Kind(InfoKind::Wireguard)]);
    mutator.nlas.push(info);
    mutator.nlas.push(Nla::IfName(IFACE_NAME.to_owned()));
    req.execute().await?;
    Ok(link_handle)
}
