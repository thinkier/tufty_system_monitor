use tokio_serial::{SerialPort, SerialPortBuilderExt};
use tokio::sync::mpsc::{channel, Sender};
use tokio::io::{AsyncWriteExt, BufStream};
use tokio::spawn;
use crate::hwstats::HwStats;

pub async fn connect_to_rp2040() -> Sender<HwStats> {
    let (tx, mut rx) = channel::<HwStats>(5);

    spawn(async move {
        let mut serial = tokio_serial::new("COM69", 9600)
            .open_native_async()
            .unwrap();
        serial.write_data_terminal_ready(true).unwrap();
        let mut buf = BufStream::new(serial);

        while let Some(x) = rx.recv().await {
            let mut bytes = serde_json::to_vec(&x).unwrap();
            bytes.push(b'\n');

            buf.write_all(&bytes).await.unwrap();
            buf.flush().await.unwrap();
        }
    }).await.unwrap();

    return tx;
}