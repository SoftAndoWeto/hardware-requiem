use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, RefreshKind, System};

use super::HwResult;

/// Represents detailed information about the CPU.
///
/// This struct is used to store and serialize CPU-related data retrieved from the system.
/// It contains the following fields:
/// - name: A string representing the brand name of the CPU.
/// - identifier: A string that uniquely identifies the CPU, typically combining vendor ID and CPU
///   name.
/// - processor_id: An optional string that can hold the processor ID, if available.
/// - vendor_frequency: A 64-bit unsigned integer representing the CPU's vendor frequency in Hz.
/// - physical_processor_count: A usize indicating the number of physical processors present in the
///   system.
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

/// Retrieves detailed information about the CPU.
///
/// This function uses the `sysinfo` crate to gather CPU-related information.
/// It creates a new `System` instance with specific refresh kinds, retrieves the first CPU, and
/// constructs a `CpuInfo` struct with the gathered data.
///
/// # Returns
///
/// A `CpuInfo` struct containing detailed information about the CPU.
pub fn get_cpu_info() -> HwResult<CpuInfo> {
    let s = System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));
    let proc = s
        .cpus()
        .first()
        .ok_or_else(|| "sysinfo returned no CPU records".to_string())?;

    Ok(CpuInfo {
        name: proc.brand().trim().to_string(),
        identifier: format!("{} - {}", proc.vendor_id(), proc.name()),
        processor_id: None,
        vendor_frequency: (1000000 * proc.frequency()),
        physical_processor_count: num_cpus::get_physical(),
    })
}
