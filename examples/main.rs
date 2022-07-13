use anyhow::Result;
use mtu::MtuV4;

#[async_std::main]
async fn main() -> Result<()> {
  // SOURCE IP ADDRESS
  // let localhost = Ipv4Addr::LOCALHOST;
  let dest = "223.5.5.5:0".parse()?;

  let timeout = 6;
  let mtu_v4 = MtuV4::new(6);

  dbg!(mtu_v4.get(dest).await);
  Ok(())
}
