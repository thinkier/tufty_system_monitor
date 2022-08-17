extern crate wmi;

use std::collections::HashMap;
use wmi::*;

#[derive(Debug, Clone, Serialize)]
pub struct SysInfo {
    pub cpu_name: String,
    pub gpu_name: String,
}

impl SysInfo {
    pub async fn load() -> Result<SysInfo, WMIError> {
        // Manufacturer, Product FROM Win32_BaseBoard
        let wmi_con = WMIConnection::new(COMLibrary::new()?.into())?;

        let gpu_name = (wmi_con.async_raw_query("SELECT Name FROM Win32_VideoController").await?
            as Vec<HashMap<String, String>>)
            .get(0)
            .map(|x| x.get("Name"))
            .flatten()
            .map(|x| x.trim().to_string())
            .unwrap_or_else(|| "GPU".to_string());
        let mut cpu_name = (wmi_con.async_raw_query("SELECT Name FROM Win32_Processor").await?
            as Vec<HashMap<String, String>>)
            .get(0)
            .map(|x| x.get("Name"))
            .flatten()
            .map(|x| x.trim().to_string())
            .unwrap_or_else(|| "CPU".to_string());

        let unwanted_suffix = "-core processor";

        if cpu_name.to_ascii_lowercase().ends_with(unwanted_suffix) {
            let trim = &cpu_name[0..(cpu_name.len() - unwanted_suffix.len())];
            cpu_name = trim.trim_end_matches(char::is_numeric).trim().to_string();
        }

        return Ok(SysInfo {
            cpu_name,
            gpu_name,
        });
    }
}
