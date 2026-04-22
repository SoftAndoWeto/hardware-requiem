use serde::{Deserialize, Serialize};

use super::HwResult;

#[cfg(any(windows, target_os = "linux"))]
mod parser;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

#[derive(Debug, Serialize, Deserialize)]
pub struct BiosInfo {
    pub uuid: String,
    pub manufacturer: String,
    pub name: String,
}

#[cfg(windows)]
pub use self::windows::get_bios_info;
#[cfg(target_os = "linux")]
pub use self::linux::get_bios_info;

#[cfg(not(any(windows, target_os = "linux")))]
pub fn get_bios_info() -> HwResult<BiosInfo> {
    Err("BIOS collection is not implemented for this platform".to_string())
}

#[cfg(test)]
mod tests;
