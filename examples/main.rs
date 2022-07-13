use std::{
  net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
  sync::Arc,
  time::Duration,
};

use mtu::MtuV4;
use pnet_packet::{
  icmp::{self},
  Packet,
};

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // SOURCE IP ADDRESS
  // let localhost = Ipv4Addr::LOCALHOST;
  let dest = "223.5.5.5:0".parse()?;

  let mtu_v4 = MtuV4::new();

  dbg!(mtu_v4.get(dest).await);
  /*
     let packet_slice = &mut [0; 1472];
     let mut buf = vec![0; 8 + 1472]; // 8 bytes of header, then payload
     let len = buf.len();
     let mut packet = icmp::echo_request::MutableEchoRequestPacket::new(&mut buf[..]).unwrap();
     packet.set_icmp_type(icmp::IcmpTypes::EchoRequest);

  // Identifier为标识符，由主机设定，一般设置为进程号，回送响应消息与回送消息中identifier保持一致 && Sequence Number为序列号，由主机设定，一般设为由0递增的序列，回送响应消息与回送消息中Sequence Number保持一致
  packet.set_identifier(2);
  packet.set_sequence_number(len as u16 - 8);
  packet.set_payload(packet_slice);

  // Calculate and set the checksum
  let icmp_packet = icmp::IcmpPacket::new(packet.packet()).unwrap();
  let checksum = icmp::checksum(&icmp_packet);
  packet.set_checksum(checksum);
  loop {
  socket_clone.send_to(&mut packet.packet(), dest).unwrap();
  std::thread::sleep(Duration::from_millis(1000));
  }
  */
  Ok(())
}
