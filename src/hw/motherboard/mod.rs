use serde::{Deserialize, Serialize};

use super::HwResult;

#[cfg(any(windows, target_os = "linux"))]
mod parser;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

/// Represents information about the motherboard.
///
/// Motherboard information retrieved from SMBIOS baseboard data.
#[derive(Debug, Serialize, Deserialize)]
pub struct MotherboardInfo {
    pub vendor: String,
    pub manufacturer: String,
    pub product: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "serialNumber")]
    pub serial_number: String,
    #[serde(rename = "assetTag")]
    pub asset_tag: Option<String>,
    #[serde(rename = "locationInChassis")]
    pub location_in_chassis: Option<String>,
    #[serde(rename = "boardType")]
    pub board_type: Option<String>,
    #[serde(rename = "featureFlags")]
    pub feature_flags: Vec<String>,
    #[serde(rename = "memorySlotCount")]
    pub memory_slot_count: Option<u16>,
    #[serde(rename = "occupiedMemorySlotCount")]
    pub occupied_memory_slot_count: Option<u16>,
    #[serde(rename = "processorSocket")]
    pub processor_socket: Option<String>,
}

#[cfg(windows)]
pub use self::windows::get_motherboard_info;
#[cfg(target_os = "linux")]
pub use self::linux::get_motherboard_info;

#[cfg(not(any(windows, target_os = "linux")))]
pub fn get_motherboard_info() -> HwResult<MotherboardInfo> {
    Err("motherboard collection is not implemented for this platform".to_string())
}

#[cfg(test)]
mod tests;
