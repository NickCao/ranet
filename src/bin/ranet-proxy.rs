use argh::FromArgs;
use log::{info, warn};
use nix::sys::socket::{setsockopt, sockopt::BindToDevice};
use shadowsocks_service::shadowsocks::relay::socks5::{
    self, Address, Command, HandshakeRequest, HandshakeResponse, TcpRequestHeader,
    TcpResponseHeader, UdpAssociateHeader,
};
use std::ffi::OsString;
use std::net::{IpAddr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::os::unix::prelude::AsRawFd;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpSocket, TcpStream, UdpSocket};

#[derive(FromArgs)]
/// ranet-proxy
struct Args {
    /// listen address (also used for UDP association)
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
    simple_logger::init_with_level(log::Level::Debug)?;
    let args: Args = argh::from_env();
    let listener = TcpListener::bind(args.listen).await?;
    info!("listening for socks5 connection on address {}", args.listen);
    while let Ok((incoming, _)) = listener.accept().await {
        tokio::spawn(process(
            incoming,
            args.listen.ip(),
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
    listen: IpAddr,
    bind: Ipv6Addr,
    interface: OsString,
    prefix: Ipv6Addr,
) -> Result<(), std::io::Error> {
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
        Command::UdpAssociate => {
            info!(
                "UDP associate from {} via {}",
                inbound.peer_addr()?,
                header.address
            );
            let client = match header.address {
                Address::SocketAddress(addr) => {
                    let client = UdpSocket::bind((listen, 0)).await?;
                    client.connect(addr).await?;
                    Ok(client)
                }
                _ => Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "client address must be sent",
                )),
            }?;
            let server = socket2::Socket::new(socket2::Domain::IPV6, socket2::Type::DGRAM, None)?;
            server.set_nonblocking(true)?;
            setsockopt(server.as_raw_fd(), BindToDevice, &interface)?;
            let bindaddr: SocketAddr = (bind, 0).into();
            server.bind(&socket2::SockAddr::from(bindaddr))?;
            let server = UdpSocket::from_std(server.into())?;
            TcpResponseHeader::new(
                socks5::Reply::Succeeded,
                Address::SocketAddress(SocketAddr::from(client.local_addr()?)),
            )
            .write_to(&mut inbound)
            .await?;
            let mut sink = tokio::io::sink();
            tokio::select!(
                res = tokio::io::copy(&mut inbound, &mut sink) => res.map(|_| ()),
                res = async {
                    let mut buf = [0u8; 65536];
                    while let Ok(n) = client.recv(&mut buf).await {
                        let data = &buf[..n];
                        let mut cur = std::io::Cursor::new(data);
                        let header = UdpAssociateHeader::read_from(&mut cur).await?;
                        if header.frag != 0 {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Unsupported,
                                "udp fragment is not supported",
                            ));
                        }
                        let pos = cur.position() as usize;
                        let payload = &data[pos..];
                        server
                            .send_to(payload, resolve(header.address, prefix).await?)
                            .await.unwrap();
                    }
                    Ok(())
                } => res,
                res = async {
                    let mut buf = [0u8; 65536];
                    while let Ok((n, peer)) = server.recv_from(&mut buf).await {
                        let data = &buf[..n];
                        let header = UdpAssociateHeader::new(0, peer.into());
                        let mut send_buf = Vec::new();
                        let mut cur = std::io::Cursor::new(&mut send_buf);
                        header.write_to(&mut cur).await?;
                        cur.write_all(data).await?;
                        client.send(&send_buf).await?;
                    }
                    Ok(())
                } => res,
            )?;
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
