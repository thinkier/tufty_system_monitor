use std::str::FromStr;
use std::string::ParseError;
use crate::SysInfo;

#[derive(Debug, Serialize)]
pub struct HwStats {
    #[serde(flatten)]
    pub info: SysInfo,
    pub temps: HwTemps,
}

#[derive(Debug, Serialize)]
pub struct HwTemps {
    cpu: Vec<i16>,
    gpu: Vec<i16>,
}

#[derive(Debug)]
pub enum Measurement {
    Rpm(u16),
    Temperature(i16),
    Percentage(f32),
    Other(String),
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