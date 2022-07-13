use std::{
  net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
  sync::Arc,
  time::Duration,
};

use anyhow::Result;
use mtu::MtuV4;
use pnet_packet::{
  icmp::{self},
  Packet,
};

#[async_std::main]
async fn main() -> Result<()> {
  // SOURCE IP ADDRESS
  // let localhost = Ipv4Addr::LOCALHOST;
  let dest = "223.5.5.5:0".parse()?;

  let mut mtu_v4 = MtuV4::new();

  dbg!(mtu_v4.get(dest).await);
  Ok(())
}
