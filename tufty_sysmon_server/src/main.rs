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
    loop {
        let (wtx, j) = port::connect_to_rp2040().await;
        hwstats::stats_watcher(wtx).await;
        j.abort();
        let _ = j.await;
    }
}
