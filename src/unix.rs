use std::{
  collections::HashMap,
  net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
  sync::Arc,
  time::Duration,
};

use async_std::{
  channel::{bounded, Receiver, Sender},
  future::timeout,
  net::UdpSocket,
  task::{spawn, JoinHandle},
};
use parking_lot::{Mutex, RwLock};
use pnet_packet::{
  icmp::{self},
  Packet,
};

use crate::UDP_HEADER_SIZE;

const RETRY: u16 = 6;
const PAYLOAD: [u8; crate::MTU_IPV4 as usize] = [9; crate::MTU_IPV4 as usize];

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn v4(buf: &[u8]) -> u16 {
  (buf.len() as u16) - crate::IPV4_HEADER_SIZE - UDP_HEADER_SIZE
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn v6(buf: &[u8]) -> u16 {
  (buf.len() as u16) - crate::IPV6_HEADER_SIZE - UDP_HEADER_SIZE
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "ios")))]
pub fn v4(buf: &[u8]) -> u16 {
  buf.len() as u16 - UDP_HEADER_SIZE
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "ios")))]
pub fn v6(buf: &[u8]) -> u16 {
  buf.len() as u16 - UDP_HEADER_SIZE
}

#[derive(Debug)]
pub struct AddrMtu {
  send: Sender<u16>,
  find: Receiver<u16>,
}

enum FindRecv {
  Find(Receiver<u16>),
  Recv(Receiver<u16>, Sender<u16>),
}

#[derive(Debug)]
pub struct MtuV4 {
  udp: std::net::UdpSocket,
  recv: Mutex<Option<JoinHandle<()>>>,
  timeout: u64,
  mtu: Arc<RwLock<HashMap<SocketAddrV4, AddrMtu>>>,
}

impl MtuV4 {
  pub fn run(&self) {
    let mut recv = self.recv.lock();
    if recv.is_some() {
      return;
    }
    let udp: UdpSocket = err::ok!(self.udp.try_clone()).unwrap().into();
    let mtu = self.mtu.clone();
    *recv = Some(spawn(async move {
      let mut buf = vec![0u8; crate::ETHERNET as usize];
      loop {
        if let Ok((recv, SocketAddr::V4(addr))) = udp.recv_from(&mut buf).await {
          //let sent = udp.send_to(&buf[..recv], &peer).await?;
          let r = {
            if let Some(addr_mtu) = mtu.read().get(&addr) {
              let len = v4(&buf[..recv]);
              Some((addr_mtu.send.clone(), len))
            } else {
              None
            }
          };
          if let Some((send, len)) = r {
            dbg!((addr, len));
            err::log!(send.send(len).await);
          }
        }
      }
    }));
  }

  pub async fn stop(&self) {
    let mut recv = self.recv.lock();
    if let Some(task) = recv.take() {
      task.cancel().await;
    }
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
      recv: Mutex::new(None),
      mtu: Arc::new(RwLock::new(HashMap::new())),
    }
  }

  pub async fn get(&self, addr: SocketAddrV4) -> u16 {
    let send_recv = {
      let mut mtu = self.mtu.write();
      if let Some(mtu) = mtu.get(&addr) {
        FindRecv::Find(mtu.find.clone())
      } else {
        let (find_s, find_r) = bounded(1);
        let (send, recv) = bounded(1);
        mtu.insert(addr, AddrMtu { find: find_r, send });
        FindRecv::Recv(recv, find_s)
      }
    };

    match send_recv {
      FindRecv::Find(find) => err::ok!(find.recv().await).unwrap(),
      FindRecv::Recv(recv, find_s) => {
        self.run();

        let udp = &self.udp;

        use crate::{MTU_IPV4, MTU_MIN_IPV4};

        icmp_v4_send(udp, addr, MTU_IPV4);

        let mut retry = RETRY;
        let mut min = MTU_MIN_IPV4;

        macro_rules! rt {
          ($len:expr) => {{
            err::log!(find_s.send($len).await);
            $len
          }};
        }

        while retry > 0 {
          let wait = Duration::from_millis(300);
          if let Ok(Ok(len)) = timeout(wait, recv.recv()).await {
            if len == MTU_IPV4 {
              return rt!(len);
            } else if len > min {
              min = len;
            }
            retry -= 1;
            continue;
          }
          break;
        }

        // 确定主机是否活着

        let mut retry = RETRY;
        let quick_ping = 100.min(self.timeout);

        while retry != 0 {
          retry -= 1;
          icmp_v4_send(udp, addr, min);
          //let wait = Duration::from_secs(self.timeout);
          let wait = Duration::from_millis(quick_ping);
          if let Ok(Ok(len)) = timeout(wait, recv.recv()).await {
            if len >= min {
              min = len;
              retry = RETRY;
              break;
            }
          }
        }

        if retry == 0 {
          let wait = Duration::from_millis(self.timeout - quick_ping);
          if let Ok(Ok(len)) = timeout(wait, recv.recv()).await {
            if len >= min {
              min = len;
              retry = RETRY;
            }
          } else {
            return rt!(0);
          }
        }

        //todo!();
        /*
           recv.recv()).await



           icmp_v4_send(udp, addr, MTU_IPV4);

           while retry > 0 {
        // err::log!(self.udp.send_to(packet.packet(), addr));
        if let Ok(r) = timeout(wait, recv.recv()).await {
        if let Ok(len) = r {
        retry = RETRY;
        dbg!((addr, len));
        }
        }
        retry -= 1;
        }
        */
        min
      }
    }
  }
}

pub fn icmp_v4_send<'p>(udp: &std::net::UdpSocket, addr: SocketAddrV4, len: u16) {
  let len = len as usize;
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
  err::log!(udp.send_to(packet.packet(), addr));
}

pub struct MtuV6 {}
