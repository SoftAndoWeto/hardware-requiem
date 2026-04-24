use serde::{Deserialize, Serialize};

use super::HwResult;

#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

#[derive(Debug, Serialize, Deserialize)]
pub struct CpuInfo {
    pub name: String,
    pub identifier: String,
    #[serde(rename = "processorId")]
    pub processor_id: Option<String>,
    #[serde(rename = "vendorFrequency")]
    pub vendor_frequency: u64,
    #[serde(rename = "physicalProcessorCount")]
    pub physical_processor_count: usize,
}

#[cfg(windows)]
pub use self::windows::get_cpu_info;
#[cfg(target_os = "linux")]
pub use self::linux::get_cpu_info;

#[cfg(not(any(windows, target_os = "linux")))]
pub fn get_cpu_info() -> HwResult<CpuInfo> {
    Err("cpu collection is not implemented for this platform".to_string())
}

fn normalize_cpu_brand(brand: &str) -> String {
    brand.trim().to_string()
}

fn compose_cpu_identifier(vendor_id: &str, cpu_name: &str) -> String {
    format!("{vendor_id} - {cpu_name}")
}

fn mhz_to_hz(mhz: u64) -> u64 {
    mhz.saturating_mul(1_000_000)
}

#[cfg(test)]
mod tests;
