extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tokio;

use crate::sysinfo::SysInfo;

mod hwstats;
mod port;
mod sysinfo;

#[tokio::main]
async fn main() {
    println!("{:?}", SysInfo::load().await);
    port::connect_to_rp2040().await;
}
