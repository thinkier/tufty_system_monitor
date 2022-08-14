use tokio_serial::{SerialPort, SerialPortBuilderExt};
use tokio::sync::watch::{channel, Sender};
use tokio::io::{AsyncWriteExt};
use tokio::spawn;
use tokio::task::JoinHandle;
use crate::hwstats::{HwStats, HwTemps};
use crate::SysInfo;

pub async fn connect_to_rp2040() -> (Sender<HwStats>, JoinHandle<()>) {
    let (tx, mut rx) = channel::<HwStats>(HwStats {
        info: SysInfo {
            cpu_name: "CPU".to_string(),
            gpu_name: "GPU".to_string(),
        },
        temps: HwTemps {
            cpu: vec![],
            gpu: vec![],
        },
    });

    let j = spawn(async move {
        let mut serial = tokio_serial::new("COM69", 9600)
            .open_native_async()
            .unwrap();
        serial.write_data_terminal_ready(true).unwrap();

        while  rx.changed().await.is_ok() {
            let mut bytes = serde_json::to_vec(&*rx.borrow()).unwrap();
            bytes.push(b'\n');

            serial.write_all(&bytes).await.unwrap();
        }
    });

    return (tx, j);
}