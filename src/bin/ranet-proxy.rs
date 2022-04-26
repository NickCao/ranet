use argh::FromArgs;
use nix::sys::socket::{setsockopt, sockopt::BindToDevice};
use shadowsocks_service::shadowsocks::relay::socks5::{
    self, Address, Command, HandshakeRequest, HandshakeResponse, TcpRequestHeader,
    TcpResponseHeader,
};
use std::ffi::OsString;
use std::net::{Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::os::unix::prelude::AsRawFd;
use tokio::net::{TcpListener, TcpSocket, TcpStream};

#[derive(FromArgs)]
/// ranet-proxy
struct Args {
    /// listen address
    #[argh(option, short = 'l')]
    listen: SocketAddr,
    /// bind address
    #[argh(option, short = 'b')]
    bind: Ipv6Addr,
    /// bind interface
    #[argh(option, short = 'i')]
    interface: OsString,
    /// nat64 prefix
    #[argh(option, short = 'p')]
    prefix: Ipv6Addr,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = argh::from_env();
    let listener = TcpListener::bind(args.listen).await?;
    while let Ok((incoming, _)) = listener.accept().await {
        tokio::spawn(process(
            incoming,
            args.bind,
            args.interface.clone(),
            args.prefix,
        ));
    }
    Ok(())
}

fn dns64(addr: SocketAddrV4, prefix: Ipv6Addr) -> SocketAddrV6 {
    let seg4 = addr.ip().to_ipv6_mapped().segments();
    let seg6 = prefix.segments();
    SocketAddrV6::new(
        Ipv6Addr::new(
            seg6[0], seg6[1], seg6[2], seg6[3], seg6[4], seg6[5], seg4[6], seg4[7],
        ),
        addr.port(),
        0,
        0,
    )
}

async fn resolve(addr: Address, prefix: Ipv6Addr) -> Result<SocketAddrV6, std::io::Error> {
    match addr {
        Address::SocketAddress(SocketAddr::V6(addr)) => Ok(addr),
        Address::SocketAddress(SocketAddr::V4(addr)) => Ok(dns64(addr, prefix)),
        Address::DomainNameAddress(domain, port) => {
            Ok(tokio::net::lookup_host((domain.as_str(), port))
                .await?
                .find_map(|addr| match addr {
                    SocketAddr::V4(a) => Some(dns64(a, prefix)),
                    SocketAddr::V6(a) => Some(a),
                })
                .ok_or(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "domain name resolves to no ip address",
                ))?)
        }
    }
}

async fn process(
    mut inbound: TcpStream,
    bind: Ipv6Addr,
    interface: OsString,
    prefix: Ipv6Addr,
) -> Result<(), std::io::Error> {
    HandshakeRequest::read_from(&mut inbound).await?;
    HandshakeResponse::new(socks5::SOCKS5_AUTH_METHOD_NONE)
        .write_to(&mut inbound)
        .await?;
    let header = TcpRequestHeader::read_from(&mut inbound).await?;
    println!(
        "INFO: connection from {} to {}",
        inbound.peer_addr()?,
        header.address
    );
    match header.command {
        Command::TcpConnect => {
            let addr = resolve(header.address, prefix).await?;
            let outbound = TcpSocket::new_v6()?;
            setsockopt(outbound.as_raw_fd(), BindToDevice, &interface)?;
            outbound.bind(SocketAddr::V6(SocketAddrV6::new(bind, 0, 0, 0)))?;
            let mut conn = outbound.connect(SocketAddr::from(addr)).await?;
            TcpResponseHeader::new(
                socks5::Reply::Succeeded,
                Address::SocketAddress(SocketAddr::from(conn.local_addr()?)),
            )
            .write_to(&mut inbound)
            .await?;
            tokio::io::copy_bidirectional(&mut inbound, &mut conn).await?;
            Ok(())
        }
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "command not implemented",
        )),
    }
}
