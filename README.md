# mtu

find mtu for windows / mac / linux , not require root permissions .

the return mtu value not include ip header + udp header len, for example if ipv4 mtu is 1500 , return mtu value is 1472

Return 0 means mtu cannot get via icmp.

base on `async_std` / `winping` .

---

查找 windows / mac / linux 的 mtu ，不需要管理员权限。

返回的 mtu 值不含 ip header + udp header 的长度，比如说 ipv4 的 mtu 1500 时，返回值是 1472。

返回 0 表示无法通过 icmp 获取 mtu。

---

[$ ./examples/main.rs](./examples/main.rs)

```rust
use std::{
  net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
  sync::Arc,
  time::Duration,
};

use pnet_packet::{
  icmp::{self},
  Packet,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // SOURCE IP ADDRESS
  // let localhost = Ipv4Addr::LOCALHOST;
  let localhost = Ipv4Addr::UNSPECIFIED;
  let socket_ip_address = SocketAddr::new(IpAddr::V4(localhost), 80);
  let socket2_ip_address = socket_ip_address.into();

  // CREATE ICMP SOCKET
  let socket2_ipv4_socket = socket2::Socket::new(
    socket2::Domain::IPV4,
    socket2::Type::DGRAM,
    Some(socket2::Protocol::ICMPV4),
  )
  .unwrap();

  // BIND TO LOCAL ADDRESS
  socket2_ipv4_socket
    .bind(&socket2_ip_address)
    .expect(&format!(
      "Failed binding to Ipv4 address {:?}",
      &socket_ip_address
    ));

  // CREATE STD SOCKET FROM SOCKET2 SOCKET
  let std_ipv4_socket: UdpSocket = socket2_ipv4_socket.into();
  std_ipv4_socket.set_read_timeout(Some(Duration::from_millis(100)))?;
  let socket_arc = Arc::new(std_ipv4_socket);
  let dest = "223.5.5.5:0";

  let socket_clone = Arc::clone(&socket_arc);
  std::thread::spawn(move || {
    let packet_slice = &mut [0; 57];
    let mut buf = vec![0; 8 + 57]; // 8 bytes of header, then payload
    let mut packet = icmp::echo_request::MutableEchoRequestPacket::new(&mut buf[..]).unwrap();
    packet.set_icmp_type(icmp::IcmpTypes::EchoRequest);
    packet.set_identifier(1);
    packet.set_sequence_number(1);
    packet.set_payload(packet_slice);

    // Calculate and set the checksum
    let icmp_packet = icmp::IcmpPacket::new(packet.packet()).unwrap();
    let checksum = icmp::checksum(&icmp_packet);
    packet.set_checksum(checksum);
    loop {
      socket_clone.send_to(&mut packet.packet(), dest).unwrap();
      std::thread::sleep(Duration::from_millis(1000));
    }
  });

  let mut buffer = [0; 1024 * 1024];
  for _ in 0..20 {
    if let Ok((bytes_read, from)) = socket_arc.recv_from(&mut buffer) {
      println!("Received {} bytes from {:?}", bytes_read, from);
      let ipv4_packet = pnet_packet::ipv4::Ipv4Packet::new(&buffer[..bytes_read]).unwrap();
      let icmp_packet = pnet_packet::icmp::IcmpPacket::new(ipv4_packet.payload()).unwrap();
      let udp_packet = pnet_packet::udp::UdpPacket::new(&ipv4_packet.payload()).unwrap();
      println!("Received {:?}", ipv4_packet);
    }
  }
  Ok(())
}
```
