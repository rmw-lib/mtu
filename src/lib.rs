#![feature(new_uninit)]

pub const ETHERNET: u16 = 1500;
pub const IPV4_HEADER_SIZE: u16 = 20;
pub const IPV6_HEADER_SIZE: u16 = 40;
pub const UDP_HEADER_SIZE: u16 = 8;
pub const MTU_IPV4: u16 = ETHERNET - IPV4_HEADER_SIZE - UDP_HEADER_SIZE;
pub const MTU_IPV6: u16 = ETHERNET - IPV6_HEADER_SIZE - UDP_HEADER_SIZE;

#[cfg(not(target_os = "windows"))]
mod unix;

#[cfg(not(target_os = "windows"))]
pub use unix::{MtuV4, MtuV6};
