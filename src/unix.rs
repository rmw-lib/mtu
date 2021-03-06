use std::{
  collections::HashMap,
  net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
  sync::{
    atomic::{AtomicU16, Ordering},
    Arc,
  },
  time::Duration,
};

use async_std::{
  channel::{bounded, Receiver, Sender},
  future::timeout,
  net::UdpSocket,
  task::{spawn, JoinHandle},
};
use parking_lot::{Mutex, RwLock};
use pnet_packet::{icmp, Packet};

use crate::{MTU_IPV4, MTU_MIN_IPV4, UDP_HEADER_SIZE};

const RETRY: u16 = 4;
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
  mtu: AtomicU16,
  channel: Arc<RwLock<HashMap<SocketAddrV4, AddrMtu>>>,
}

impl MtuV4 {
  pub fn run(&self) {
    let mut recv = self.recv.lock();
    if recv.is_some() {
      return;
    }
    let udp: UdpSocket = err::ok!(self.udp.try_clone()).unwrap().into();
    let channel = self.channel.clone();
    *recv = Some(spawn(async move {
      let mut buf = vec![0u8; crate::ETHERNET as usize];
      loop {
        if let Ok((recv, SocketAddr::V4(addr))) = udp.recv_from(&mut buf).await {
          //let sent = udp.send_to(&buf[..recv], &peer).await?;
          let r = {
            if let Some(addr_mtu) = channel.read().get(&addr) {
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
    let task = { self.recv.lock().take() };
    if let Some(task) = task {
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
      mtu: AtomicU16::new(MTU_MIN_IPV4),
      recv: Mutex::new(None),
      channel: Arc::new(RwLock::new(HashMap::new())),
    }
  }

  pub async fn get(&self, addr: SocketAddrV4) -> u16 {
    let send_recv = {
      let mut channel = self.channel.write();
      if let Some(mtu) = channel.get(&addr) {
        FindRecv::Find(mtu.find.clone())
      } else {
        let (find_s, find_r) = bounded(1);
        let (send, recv) = bounded(1);
        channel.insert(addr, AddrMtu { find: find_r, send });
        FindRecv::Recv(recv, find_s)
      }
    };

    match send_recv {
      FindRecv::Find(find) => err::ok!(find.recv().await).unwrap(),
      FindRecv::Recv(recv, find_s) => {
        self.run();

        let udp = &self.udp;

        macro_rules! rt {
          ($len:expr) => {{
            err::log!(find_s.send($len).await);

            let mtu = self.mtu.load(Ordering::Relaxed);
            if mtu != $len {
              if mtu > $len {
                self.mtu.fetch_sub((mtu - $len) / 2, Ordering::Relaxed);
              } else {
                self.mtu.fetch_add(($len - mtu) / 2, Ordering::Relaxed);
              }
            }

            $len
          }};
        }

        macro_rules! send {
          ($len:expr) => {
            icmp_v4_send(udp, addr, $len);
          };
        }

        macro_rules! wait {
          ($time:expr) => {{
            let wait = Duration::from_millis($time);
            timeout(wait, recv.recv()).await
          }};
        }

        let mut mtu = MTU_IPV4;
        send!(mtu);

        let mut min = MTU_MIN_IPV4;
        let quick_ping = 200.min(self.timeout);

        if let Ok(Ok(len)) = wait!(quick_ping) {
          if len == MTU_IPV4 {
            return rt!(len);
          } else if len > min {
            min = len;
          }
        }

        // ????????????????????????

        let mut retry = RETRY;

        while retry != 0 {
          retry -= 1;
          send!(min);
          if let Ok(Ok(len)) = wait!(quick_ping) {
            if len == MTU_IPV4 {
              return rt!(len);
            }
            if len >= min {
              min = len;
              retry = RETRY;
              break;
            }
          }
        }

        let mut timeout = self.timeout;
        if retry == 0 {
          if let Ok(Ok(len)) = wait!(timeout - quick_ping) {
            if len == MTU_IPV4 {
              return rt!(len);
            }
            if len >= min {
              min = len;
              retry = RETRY;
            }
          } else {
            return rt!(0);
          }
        }

        let mut step = 16;

        loop {
          mtu -= step;
          if mtu < min {
            break;
          }

          send!(mtu);

          if let Ok(Ok(len)) = wait!(quick_ping) {
            if len == MTU_IPV4 {
              return rt!(len);
            }
            if len >= mtu {
              min = len;
              break;
            }
          }
        }

        while step > 1 {
          step /= 2;
          mtu = min + step;
          send!(mtu);

          if let Ok(Ok(len)) = wait!(quick_ping) {
            if len == MTU_IPV4 {
              return rt!(len);
            }
            if len > min {
              min = len;
            }
          }
        }

        while timeout >= quick_ping && retry != 0 {
          send!(min + 1);
          let t = (min + MTU_IPV4) / 2;
          if t > min {
            send!(t);
          }
          timeout -= quick_ping;
          if let Ok(Ok(len)) = wait!(quick_ping) {
            if len == MTU_IPV4 {
              return rt!(len);
            }
            if len > min {
              retry = RETRY;
              timeout = self.timeout;
              min = len;
              continue;
            }
          }
          retry -= 1;
        }

        min
      }
    }
  }
}

pub fn icmp_v4_send(udp: &std::net::UdpSocket, addr: SocketAddrV4, len: u16) {
  let len = len as usize;
  let mut buf = unsafe { Box::<[u8]>::new_uninit_slice(8 + len).assume_init() };
  let mut packet = icmp::echo_request::MutableEchoRequestPacket::new(&mut buf[..]).unwrap();
  packet.set_icmp_type(icmp::IcmpTypes::EchoRequest);

  // Identifier????????????????????????????????????????????????????????????????????????????????????????????????identifier???????????? && Sequence Number????????????????????????????????????????????????0??????????????????????????????????????????????????????Sequence Number????????????
  // linux ?????? Identifier ???????????????????????????udp?????????
  //packet.set_identifier();

  packet.set_sequence_number(len as u16);
  packet.set_payload(&PAYLOAD[..len]);

  let icmp_packet = icmp::IcmpPacket::new(packet.packet()).unwrap();
  let checksum = icmp::checksum(&icmp_packet);
  packet.set_checksum(checksum);
  err::log!(udp.send_to(packet.packet(), addr));
}

pub struct MtuV6 {}
