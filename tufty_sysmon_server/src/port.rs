use std::error::Error;
use std::time::Duration;
use tokio_serial::{SerialPort, SerialPortBuilderExt};
use tokio::sync::watch::{channel, Receiver, Sender};
use tokio::io::{AsyncWriteExt};
use tokio::spawn;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use crate::hwstats::{HwStats};

pub async fn connect_to_rp2040() -> (Sender<Option<HwStats>>, JoinHandle<()>) {
    let (tx, mut rx) = channel::<Option<HwStats>>(None);

    let j = spawn(async move {
        loop {
            if let Err(e) = rp2040_comms_manager(&mut rx).await {
                eprintln!("Serial port comms error: {:?}", e);
            }
            sleep(Duration::from_secs(1)).await;
        }
    });

    return (tx, j);
}

async fn rp2040_comms_manager(rx: &mut Receiver<Option<HwStats>>) -> Result<(), Box<dyn Error>> {
    let mut serial = tokio_serial::new("COM69", 9600)
        .open_native_async()?;
    serial.write_data_terminal_ready(true)?;

    while rx.changed().await.is_ok() {
        serial.write_all(&{
            let s: &Option<HwStats> = &*(*rx).borrow();

            let mut bytes = serde_json::to_vec(s)?;
            bytes.push(b'\n');
            bytes
        }).await?;
    }

    Ok(())
}