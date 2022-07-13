use anyhow::Result;
use mtu::MtuV4;

#[async_std::main]
async fn main() -> Result<()> {
  // SOURCE IP ADDRESS
  // let localhost = Ipv4Addr::LOCALHOST;
  let dest = "103.74.50.106:0".parse()?;

  let timeout = 6000;
  let mtu_v4 = MtuV4::new(timeout);

  dbg!(mtu_v4.get(dest).await);
  Ok(())
}
