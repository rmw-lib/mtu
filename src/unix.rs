use std::{
  net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
  time::Duration,
};

use async_std::{
  future::{pending, timeout},
  net::UdpSocket,
  task::{spawn, JoinHandle},
};
use pnet_packet::{
  icmp::{self},
  Packet,
};

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

#[derive(Debug)]
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
          dbg!(111);
          //let sent = udp.send_to(&buf[..recv], &peer).await?;
          println!("{} -> {}", peer, v4(&buf[..recv]));
        }
        dbg!("end");
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

  pub async fn get(&mut self, addr: SocketAddrV4) -> u16 {
    let len = 1472;
    let mut buf = unsafe { Box::<[u8]>::new_uninit_slice(8 + len).assume_init() };
    let payload = unsafe { Box::<[u8]>::new_uninit_slice(len).assume_init() };

    let len = buf.len();
    let mut packet = icmp::echo_request::MutableEchoRequestPacket::new(&mut buf[..]).unwrap();
    packet.set_icmp_type(icmp::IcmpTypes::EchoRequest);

    // Identifier为标识符，由主机设定，一般设置为进程号，回送响应消息与回送消息中identifier保持一致 && Sequence Number为序列号，由主机设定，一般设为由0递增的序列，回送响应消息与回送消息中Sequence Number保持一致
    packet.set_identifier(2);
    packet.set_sequence_number(len as u16 - 8);
    packet.set_payload(&payload);

    let icmp_packet = icmp::IcmpPacket::new(packet.packet()).unwrap();
    let checksum = icmp::checksum(&icmp_packet);
    packet.set_checksum(checksum);

    self.run();
    err::log!(self.udp.send_to(packet.packet(), addr));

    let mut buf = [0; 1500];
    let never = pending::<()>();
    let dur = Duration::from_secs(5);

    err::log!(timeout(dur, never).await);
    0
  }
}

pub struct MtuV6 {}
