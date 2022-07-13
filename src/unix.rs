pub use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn v4<'a>(buf: &'a [u8]) -> u16 {
  (buf.len() as u16) - crate::IPV4_HEADER_SIZE
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn v6<'a>(buf: &'a [u8]) -> u16 {
  (buf.len() as u16) - crate::IPV6_HEADER_SIZE
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "ios")))]
pub fn v4<'a>(buf: &'a [u8]) -> u16 {
  buf.len() as u16
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "ios")))]
pub fn v6<'a>(buf: &'a [u8]) -> u16 {
  buf.len() as u16
}

pub struct MtuV6 {}

pub struct MtuV4 {}

impl MtuV4 {
  pub fn new() -> Self {
    let localhost = Ipv4Addr::UNSPECIFIED;
    let socket = err::ok!(socket2::Socket::new(
      socket2::Domain::IPV4,
      socket2::Type::DGRAM,
      Some(socket2::Protocol::ICMPV4),
    ))
    .unwrap();
    err::log!(socket.bind(&SocketAddr::new(IpAddr::V4(localhost), 0).into()));
    Self {}
  }

  pub async fn get(&self, addr: SocketAddrV4) -> u16 {
    //self.ing.insert(SocketAddrV4)
    0
  }
}
