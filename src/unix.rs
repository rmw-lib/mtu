pub use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};

pub use async_std::net::UdpSocket;
use async_std::task::{spawn, JoinHandle};

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn v4(buf: &[u8]) -> u16 {
  (buf.len() as u16) - crate::IPV4_HEADER_SIZE
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn v6(buf: &[u8]) -> u16 {
  (buf.len() as u16) - crate::IPV6_HEADER_SIZE
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "ios")))]
pub fn v4(buf: &[u8]) -> u16 {
  buf.len() as u16
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "ios")))]
pub fn v6(buf: &[u8]) -> u16 {
  buf.len() as u16
}

pub struct MtuV4 {
  udp: std::net::UdpSocket,
  run: Option<JoinHandle<()>>,
}

impl MtuV4 {
  pub fn run(&mut self) {
    if self.run.is_some() {
      return;
    }
    let udp: UdpSocket = err::ok!(self.udp.try_clone()).unwrap().into();
    self.run = Some(spawn(async move {
      let mut buf = vec![0u8; crate::ETHERNET as usize];
      loop {
        if let Ok((recv, peer)) = udp.recv_from(&mut buf).await {
          //let sent = udp.send_to(&buf[..recv], &peer).await?;
          println!("{} -> {}", peer, v4(&buf[..recv]));
        }
      }
    }));
  }

  pub fn new() -> Self {
    let localhost = Ipv4Addr::UNSPECIFIED;
    let udp = err::ok!(socket2::Socket::new(
      socket2::Domain::IPV4,
      socket2::Type::DGRAM,
      Some(socket2::Protocol::ICMPV4),
    ))
    .unwrap();
    err::log!(udp.bind(&SocketAddr::new(IpAddr::V4(localhost), 0).into()));
    let udp: std::net::UdpSocket = udp.into();
    let mut me = Self { udp, run: None };
    me.run();
    me
  }

  pub async fn get(&self, addr: SocketAddrV4) -> u16 {
    //self.ing.insert(SocketAddrV4)
    0
  }
}

pub struct MtuV6 {}
