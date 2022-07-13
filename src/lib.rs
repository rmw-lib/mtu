pub const ETHERNET: u16 = 1500;
pub const IPV4_HEADER_SIZE: u16 = 20;
pub const IPV6_HEADER_SIZE: u16 = 40;
pub const UDP_HEADER_SIZE: u16 = 8;
pub const UDP_IPV4: u16 = ETHERNET - IPV4_HEADER_SIZE - UDP_HEADER_SIZE;
pub const UDP_IPV6: u16 = ETHERNET - IPV6_HEADER_SIZE - UDP_HEADER_SIZE;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn v4<'a>(buf: &'a [u8]) -> Option<(u16, u16)> {
  use pnet_packet::{icmp, Packet};
  if let Some(packet) = pnet_packet::ipv4::Ipv4Packet::new(&buf) {
    if let Some(n) = icmp::echo_reply::EchoReplyPacket::new(packet.payload()) {
      return Some((n.get_identifier(), n.get_sequence_number()));
    }
  }
  None
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn v6<'a>(buf: &'a [u8]) -> Option<(u16, u16)> {
  use pnet_packet::{icmpv6, Packet};
  if let Some(packet) = pnet_packet::ipv6::Ipv6Packet::new(&buf) {
    if let Some(n) = icmpv6::echo_reply::EchoReplyPacket::new(packet.payload()) {
      return Some((n.get_identifier(), n.get_sequence_number()));
    }
  }
  None
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "ios")))]
pub fn v4<'a>(buf: &'a [u8]) -> Option<(u16, u16)> {
  use pnet_packet::icmp;
  if let Some(n) = icmp::echo_reply::EchoReplyPacket::new(&buf) {
    return Some((n.get_identifier(), n.get_sequence_number()));
  }
  None
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "ios")))]
pub fn v6<'a>(buf: &'a [u8]) -> Option<(u16, u16)> {
  use pnet_packet::icmpv6;
  if let Some(n) = icmpv6::echo_reply::EchoReplyPacket::new(&buf) {
    return Some((n.get_identifier(), n.get_sequence_number()));
  }
  None
}
