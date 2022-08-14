use std::error::Error;
use std::fs;
use std::str::FromStr;
use std::string::ParseError;
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;
use tokio::fs as tfs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::spawn;
use tokio::sync::watch::{Sender as WSender};
use tokio::time::sleep;
use crate::SysInfo;

#[derive(Debug, Serialize)]
pub struct HwStats {
    #[serde(flatten)]
    pub info: SysInfo,
    pub temps: HwTemps,
}

#[derive(Debug, Clone, Serialize)]
pub struct HwTemps {
    pub cpu: Vec<i16>,
    pub gpu: Vec<i16>,
}

#[derive(Debug)]
pub enum Measurement {
    Rpm(u16),
    Temperature(i16),
    Percentage(f32),
    Other(String),
}

impl Measurement {
    pub fn as_i16(&self) -> i16 {
        if let Measurement::Temperature(ref i) = self {
            return *i;
        }

        panic!("cannot convert to i16 from {:?}", self);
    }
}

impl FromStr for Measurement {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.ends_with("RPM") {
            let rpm = s.trim_end_matches("RPM").parse::<u16>().unwrap();
            Ok(Measurement::Rpm(rpm))
        } else if s.ends_with("°C") {
            let temp = s.trim_end_matches("°C").parse::<f32>().unwrap();
            Ok(Measurement::Temperature((temp * 1e1).round() as i16))
        } else if s.ends_with('%') {
            let percentage = s.trim_end_matches('%').parse::<f32>().unwrap();
            Ok(Measurement::Percentage(percentage / 1e2))
        } else {
            Ok(Measurement::Other(s.to_string()))
        }
    }
}

fn get_cpu_and_gpu_temp_pos_by_colname(str: &str) -> (usize, usize) {
    let mut cpu = 0;
    let mut gpu = 0;

    str.split(",")
        .into_iter()
        .map(|s| s.trim_matches('"'))
        .enumerate()
        .filter_map(|(i, s)| if s.ends_with(" Package") {
            Some((i, s.strip_suffix(" Package").unwrap_or(s)))
        } else {
            None
        })
        .for_each(|(i, x)| match x {
            "CPU" => cpu = i,
            "GPU" => gpu = i,
            _ => {}
        });

    return (cpu, gpu);
}

pub async fn stats_watcher(wtx: WSender<HwStats>) {
    let info = SysInfo::load().await.unwrap();

    loop {
        let (tx, rx) = channel();
        let h = spawn(async {
            let _ = read_from_icue_log(tx).await;
        });

        'reader: loop {
            // 5 second timeout
            let mut ticks = 100;

            loop {
                if let Ok(temps) = rx.try_recv() {
                    let send = wtx.send(HwStats {
                        info: info.clone(),
                        temps,
                    }).is_ok();

                    if !send {
                        h.abort();
                        let _ = h.await;
                        return;
                    }

                    break;
                }

                ticks -= 1;
                sleep(Duration::from_millis(50)).await;

                if ticks <= 0 {
                    break 'reader;
                }
            }
        }
    }
}

async fn read_from_icue_log(tx: Sender<HwTemps>) -> Result<(), Box<dyn Error>> {
    let mut paths = fs::read_dir("%homedrive%%homepath%\\Documents\\iCUE")?
        .into_iter()
        .flat_map(|x| x.ok())
        .map(|x| x.path())
        .collect::<Vec<_>>();

    paths.sort_unstable();
    if let Some(path) = paths.pop() {
        let mut file = BufReader::new(tfs::OpenOptions::new()
            .read(true)
            .open(path)
            .await?);

        let mut line = String::new();
        file.read_line(&mut line).await?;
        let (cpu_i, gpu_i) = get_cpu_and_gpu_temp_pos_by_colname(&line);

        let mut cpu = vec![];
        let mut gpu = vec![];

        loop {
            line.clear();
            file.read_line(&mut line).await?;
            let items = line.split(",").collect::<Vec<_>>();
            cpu.push(items[cpu_i].parse::<Measurement>()?.as_i16());
            gpu.push(items[gpu_i].parse::<Measurement>()?.as_i16());

            if cpu.len() > 60 {
                cpu.remove(0);
            }

            if gpu.len() > 60 {
                gpu.remove(0);
            }

            tx.send(HwTemps {
                cpu: cpu.clone(),
                gpu: gpu.clone(),
            })?;
        }
    }

    Ok(())
}