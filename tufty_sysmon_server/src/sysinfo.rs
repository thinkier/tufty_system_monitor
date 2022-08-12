extern crate wmi;

use std::collections::HashMap;
use wmi::*;

#[derive(Debug, Serialize)]
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
        let cpu_name = (wmi_con.async_raw_query("SELECT Name FROM Win32_Processor").await?
            as Vec<HashMap<String, String>>)
            .get(0)
            .map(|x| x.get("Name"))
            .flatten()
            .map(|x| x.trim().to_string())
            .unwrap_or_else(|| "CPU".to_string());

        return Ok(SysInfo {
            cpu_name,
            gpu_name,
        });
    }
}
