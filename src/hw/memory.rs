use serde::{Deserialize, Serialize};
use smbioslib::{table_load_from_device, SMBiosMemoryDevice};

use super::HwResult;

/// Represents information about a memory device in the system.
///
/// This struct is used to store and serialize data related to an individual memory device
/// as retrieved from the system's SMBIOS tables. It contains the following fields:
/// - memory_type: A string representing the type of memory (e.g., "DDR4").
/// - capacity: An unsigned 16-bit integer representing the memory capacity in megabytes.
/// - clock_speed: An unsigned 16-bit integer representing the memory clock speed in megahertz.
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryInfo {
    #[serde(rename = "type")]
    pub memory_type: String,
    pub capacity: u16,
    #[serde(rename = "clockSpeed")]
    pub clock_speed: u16,
}

/// Retrieves a list of memory information from the system's SMBIOS tables.
///
/// This function is only available on Windows systems. It uses the `smbioslib` crate to parse the
/// SMBIOS tables and extract relevant information about installed memory devices.
///
/// # Returns
///
/// A vector of `MemoryInfo` structs, each containing the following fields:
/// - `memory_type`: A string representing the type of memory (e.g., "DDR4").
/// - `capacity`: An unsigned 16-bit integer representing the memory capacity in megabytes.
/// - `clock_speed`: An unsigned 16-bit integer representing the memory clock speed in megahertz.
#[cfg(target_os = "windows")]
pub fn get_memory_info() -> HwResult<Vec<MemoryInfo>> {
    let data = table_load_from_device().map_err(|error| error.to_string())?;
    let mut mem_info_list = vec![];

    for memory_device in data.collect::<SMBiosMemoryDevice>() {
        let clock_speed = match memory_device.speed() {
            Some(msd) => match msd {
                smbioslib::MemorySpeed::Unknown => continue,
                smbioslib::MemorySpeed::SeeExtendedSpeed => continue,
                smbioslib::MemorySpeed::MTs(speed) => speed,
            },
            None => continue,
        };

        let size = match memory_device.size() {
            Some(size) => match size {
                smbioslib::MemorySize::NotInstalled => continue,
                smbioslib::MemorySize::Unknown => continue,
                smbioslib::MemorySize::SeeExtendedSize => continue,
                smbioslib::MemorySize::Kilobytes(kb_size) => kb_size / 1024,
                smbioslib::MemorySize::Megabytes(mb_size) => mb_size,
            },
            None => continue,
        };

        let memory_type = match memory_device.memory_type() {
            Some(memory_type) => format!("{:?}", memory_type.value),
            None => continue,
        };

        mem_info_list.push(MemoryInfo {
            memory_type,
            capacity: size,
            clock_speed,
        });
    }
    Ok(mem_info_list)
}
