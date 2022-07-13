use std::{
  net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
  time::Duration,
};

use async_std::{
  future::{pending, timeout},
  net::UdpSocket,
  task::{spawn, JoinHandle},
};
use dashmap::DashMap;
use parking_lot::RwLock;
use pnet_packet::{
  icmp::{self},
  Packet,
};

const PAYLOAD: [u8; crate::MTU_IPV4 as usize] = [9; crate::MTU_IPV4 as usize];

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
  recv: RwLock<Option<JoinHandle<()>>>,
  timeout: u64,
  mtu: DashMap<SocketAddrV4, (u16, u16)>,
}

impl MtuV4 {
  pub fn run(&self) {
    {
      if self.recv.read().is_some() {
        return;
      }
    }
    let udp: UdpSocket = err::ok!(self.udp.try_clone()).unwrap().into();
    *self.recv.write() = Some(spawn(async move {
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

  pub fn new(timeout: u64) -> Self {
    let ip = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
    let addr = SocketAddr::new(ip, 0);

    /*
    let port = {
    std::net::UdpSocket::bind(addr)
    .unwrap()
    .local_addr()
    .unwrap()
    .port()
    };

    let addr = SocketAddr::new(ip, port);
    */

    let udp = err::ok!(socket2::Socket::new(
      socket2::Domain::IPV4,
      socket2::Type::DGRAM,
      Some(socket2::Protocol::ICMPV4),
    ))
    .unwrap();
    err::log!(udp.bind(&addr.into()));
    let udp: std::net::UdpSocket = udp.into();
    Self {
      udp,
      timeout,
      recv: RwLock::new(None),
      mtu: DashMap::<SocketAddrV4, (u16, u16)>::new(),
    }
  }

  pub async fn get(&self, addr: SocketAddrV4) -> u16 {
    let len = 1472;
    let mut buf = unsafe { Box::<[u8]>::new_uninit_slice(8 + len).assume_init() };

    let mut packet = icmp::echo_request::MutableEchoRequestPacket::new(&mut buf[..]).unwrap();
    packet.set_icmp_type(icmp::IcmpTypes::EchoRequest);

    // Identifier为标识符，由主机设定，一般设置为进程号，回送响应消息与回送消息中identifier保持一致 && Sequence Number为序列号，由主机设定，一般设为由0递增的序列，回送响应消息与回送消息中Sequence Number保持一致
    // linux 貌似 Identifier 设置无效，会设置成udp的端口
    //packet.set_identifier();

    packet.set_sequence_number(len as u16);
    packet.set_payload(&PAYLOAD[..len]);

    let icmp_packet = icmp::IcmpPacket::new(packet.packet()).unwrap();
    let checksum = icmp::checksum(&icmp_packet);
    packet.set_checksum(checksum);

    self.run();
    err::log!(self.udp.send_to(packet.packet(), addr));

    let never = pending::<()>();
    let dur = Duration::from_secs(self.timeout);

    err::log!(timeout(dur, never).await);
    0
  }
}

pub struct MtuV6 {}
