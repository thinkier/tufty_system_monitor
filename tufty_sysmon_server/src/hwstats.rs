use std::error::Error;
use std::{env, fs};
use std::collections::VecDeque;
use std::path::PathBuf;
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

const TEMPS_LENGTH_CAP: usize = 60;

#[derive(Debug, Serialize)]
pub struct HwStats {
    #[serde(flatten)]
    pub info: SysInfo,
    pub time: String,
    #[serde(flatten)]
    pub temps: HwTemps,
}

#[derive(Debug, Clone, Serialize)]
pub struct HwTemps {
    pub cpu_temps: VecDeque<i16>,
    pub gpu_temps: VecDeque<i16>,
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

pub async fn stats_watcher(wtx: WSender<Option<HwStats>>) {
    let info = SysInfo::load().await.unwrap();

    loop {
        let (tx, rx) = channel();
        let h = spawn(async move {
            read_from_icue_log(tx).await.unwrap();
        });

        'reader: loop {
            // 5 second timeout
            let mut ticks = 100;

            loop {
                if let Ok(temps) = rx.try_recv() {
                    let send = wtx.send(Some(HwStats {
                        info: info.clone(),
                        time: chrono::Local::now().naive_local().format("%H:%M").to_string(),
                        temps,
                    })).is_ok();

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
    let mut path = PathBuf::from(env::var("USERPROFILE")?);
    path.push("Documents");
    path.push("iCUE");

    let mut paths = fs::read_dir(path)?
        .into_iter()
        .flat_map(|x| x.ok())
        .map(|x| x.path())
        .collect::<Vec<_>>();

    paths.sort_unstable();
    if let Some(path) = paths.pop() {
        let mut lim: i64 = -(tfs::metadata(&path).await?.len() as i64);
        let mut file = BufReader::new(tfs::OpenOptions::new()
            .read(true)
            .open(&path)
            .await?);

        let mut line = String::new();
        lim += file.read_line(&mut line).await? as i64;
        let (cpu_i, gpu_i) = get_cpu_and_gpu_temp_pos_by_colname(&line);

        let mut cpu = VecDeque::new();
        let mut gpu = VecDeque::new();

        loop {
            line.clear();
            let c = file.read_line(&mut line).await?;
            lim += c as i64;
            if c == 0 {
                sleep(Duration::from_millis(50)).await;
                continue;
            }
            let items = line.split(",").collect::<Vec<_>>();
            cpu.push_back(items[cpu_i].parse::<Measurement>()?.as_i16());
            gpu.push_back(items[gpu_i].parse::<Measurement>()?.as_i16());

            while cpu.len() > TEMPS_LENGTH_CAP {
                cpu.pop_front();
            }

            while gpu.len() > TEMPS_LENGTH_CAP {
                gpu.pop_front();
            }

            if lim > 0 {
                tx.send(HwTemps {
                    cpu_temps: cpu.clone(),
                    gpu_temps: gpu.clone(),
                })?;
            }
        }
    }

    Ok(())
}