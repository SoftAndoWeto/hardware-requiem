//! Hardware information collection.
//!
//! The public API is intentionally small while the platform-specific code lives
//! behind `cfg` modules. Start with [`get_hardware_info`] and expand the typed
//! records as new collectors are added.

mod error;
pub mod hw;
mod platform;
mod types;

pub use error::{HardwareInfoError, Result};
pub use types::{CpuArchitecture, CpuInfo, DiskInfo, DiskKind, HardwareInfo, MemoryInfo, OsInfo};

/// Collects a snapshot of hardware and operating system information.
pub fn get_hardware_info() -> Result<HardwareInfo> {
    platform::collect()
}
