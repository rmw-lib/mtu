pub const ETHERNET: u16 = 1500;
pub const IPV4_HEADER_SIZE: u16 = 20;
pub const IPV6_HEADER_SIZE: u16 = 40;
pub const UDP_HEADER_SIZE: u16 = 8;
pub const UDP_IPV4: u16 = ETHERNET - IPV4_HEADER_SIZE - UDP_HEADER_SIZE;
pub const UDP_IPV6: u16 = ETHERNET - IPV6_HEADER_SIZE - UDP_HEADER_SIZE;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn v4<'a>(buf: &'a [u8]) -> u16 {
  (buf.len() as u16) - IPV4_HEADER_SIZE
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn v6<'a>(buf: &'a [u8]) -> u16 {
  (buf.len() as u16) - IPV6_HEADER_SIZE
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "ios")))]
pub fn v4<'a>(buf: &'a [u8]) -> u16 {
  buf.len() as u16
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "ios")))]
pub fn v6<'a>(buf: &'a [u8]) -> u16 {
  buf.len() as u16
}
