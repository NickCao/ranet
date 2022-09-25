use argh::FromArgs;
use log::{info, warn};
use nix::sys::socket::{setsockopt, sockopt::BindToDevice};
use shadowsocks_service::shadowsocks::relay::socks5::{
    self, Address, Command, HandshakeRequest, HandshakeResponse, TcpRequestHeader,
    TcpResponseHeader,
};
use std::ffi::OsString;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::os::unix::prelude::AsRawFd;
use tokio::net::{TcpListener, TcpSocket, TcpStream};

#[derive(FromArgs, Clone)]
/// ranet-proxy
struct Args {
    /// listen address (also used for UDP association)
    #[argh(option, short = 'l')]
    listen: SocketAddr,
    /// ipv4 bind address
    #[argh(option)]
    bind4: Option<Ipv4Addr>,
    /// ipv6 bind address
    #[argh(option)]
    bind6: Option<Ipv6Addr>,
    /// bind interface
    #[argh(option, short = 'i')]
    interface: Option<OsString>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(log::Level::Debug)?;
    let args: Args = argh::from_env();
    let listener = TcpListener::bind(args.listen).await?;
    info!("listening for socks5 connection on address {}", args.listen);
    while let Ok((incoming, _)) = listener.accept().await {
        let args = args.clone();
        tokio::spawn(async move {
            if let Err(err) = process(incoming, args.clone()).await {
                warn!("{}", err);
            }
        });
    }
    Ok(())
}

async fn resolve(addr: Address) -> Result<SocketAddr, std::io::Error> {
    match addr {
        Address::SocketAddress(addr) => Ok(addr),
        Address::DomainNameAddress(domain, port) => {
            Ok(tokio::net::lookup_host((domain.as_str(), port))
                .await?
                .find_map(|addr| Some(addr))
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "domain name resolves to no ip address",
                    )
                })?)
        }
    }
}

async fn process(mut inbound: TcpStream, args: Args) -> Result<(), std::io::Error> {
    // handshake
    HandshakeRequest::read_from(&mut inbound).await?;
    HandshakeResponse::new(socks5::SOCKS5_AUTH_METHOD_NONE)
        .write_to(&mut inbound)
        .await?;
    let header = TcpRequestHeader::read_from(&mut inbound).await?;
    // command
    match header.command {
        Command::TcpConnect => {
            info!(
                "TCP connect from {} to {}",
                inbound.peer_addr()?,
                header.address
            );
            let addr = resolve(header.address).await?;
            let outbound = match addr {
                SocketAddr::V4(_) => TcpSocket::new_v4()?,
                SocketAddr::V6(_) => TcpSocket::new_v6()?,
            };
            if let Some(interface) = args.interface {
                setsockopt(outbound.as_raw_fd(), BindToDevice, &interface)?;
            }
            match addr {
                SocketAddr::V4(_) => {
                    if let Some(bind) = args.bind4 {
                        outbound.bind(SocketAddr::V4(SocketAddrV4::new(bind, 0)))?;
                    }
                }
                SocketAddr::V6(_) => {
                    if let Some(bind) = args.bind6 {
                        outbound.bind(SocketAddr::V6(SocketAddrV6::new(bind, 0, 0, 0)))?;
                    }
                }
            };
            let mut conn = outbound.connect(SocketAddr::from(addr)).await?;
            TcpResponseHeader::new(
                socks5::Reply::Succeeded,
                Address::SocketAddress(conn.local_addr()?),
            )
            .write_to(&mut inbound)
            .await?;
            tokio::io::copy_bidirectional(&mut inbound, &mut conn).await?;
            Ok(())
        }
        cmd => {
            warn!(
                "unsupported command {:?} from {}",
                cmd,
                inbound.peer_addr()?
            );
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "command not implemented",
            ))
        }
    }
}
