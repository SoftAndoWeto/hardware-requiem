use serde::{Deserialize, Serialize};

use super::HwResult;

#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

/// Represents information about a storage device.
#[derive(Debug, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub model: String,
    #[serde(rename = "serialNumber")]
    pub serial_number: String,
    pub size: u64,
}

#[cfg(windows)]
pub use self::windows::{get_logical_storage, get_storage};
#[cfg(target_os = "linux")]
pub use self::linux::get_storage;

#[cfg(not(any(windows, target_os = "linux")))]
pub fn get_storage() -> HwResult<Vec<DiskInfo>> {
    Err("storage collection is not implemented for this platform".to_string())
}

#[cfg(test)]
mod tests;
