use serde::{Deserialize, Serialize};

use super::HwResult;
#[cfg(target_os = "windows")]
use super::smbios::{parse_smbios_structures, read_raw_smbios_table, smbios_table_bytes};

/// Represents information about a memory device in the system.
///
/// This struct is used to store and serialize data related to an individual memory device
/// as retrieved from the system's SMBIOS tables. It contains the following fields:
/// - memory_type: A string representing the type of memory (e.g., "DDR4").
/// - capacity: An unsigned 16-bit integer representing the memory capacity in megabytes.
/// - clock_speed: An unsigned 16-bit integer representing the memory clock speed in megahertz.
/// - vendor/manufacturer and module identifiers when SMBIOS provides them.
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryInfo {
    #[serde(rename = "type")]
    pub memory_type: String,
    pub capacity: u16,
    #[serde(rename = "clockSpeed")]
    pub clock_speed: u16,
    pub vendor: Option<String>,
    pub manufacturer: Option<String>,
    #[serde(rename = "serialNumber")]
    pub serial_number: Option<String>,
    #[serde(rename = "partNumber")]
    pub part_number: Option<String>,
    #[serde(rename = "deviceLocator")]
    pub device_locator: Option<String>,
    #[serde(rename = "bankLocator")]
    pub bank_locator: Option<String>,
}

/// Retrieves a list of memory information from SMBIOS type 17 structures.
#[cfg(target_os = "windows")]
pub fn get_memory_info() -> HwResult<Vec<MemoryInfo>> {
    let smbios = read_raw_smbios_table()?;
    parse_memory_info_from_smbios(&smbios)
}

#[cfg(not(target_os = "windows"))]
pub fn get_memory_info() -> HwResult<Vec<MemoryInfo>> {
    Err("memory collection is only implemented on Windows".to_string())
}

#[cfg(target_os = "windows")]
fn parse_memory_info_from_smbios(raw_smbios: &[u8]) -> HwResult<Vec<MemoryInfo>> {
    let table = smbios_table_bytes(raw_smbios)?;
    let structures = parse_smbios_structures(table);
    let mut memory = Vec::new();

    for structure in structures.iter().filter(|structure| structure.structure_type == 17) {
        let Some(size_mb) = memory_device_size_mb(structure) else {
            continue;
        };
        let Some(clock_speed) = memory_device_speed_mts(structure) else {
            continue;
        };
        let Some(capacity) = u16::try_from(size_mb).ok() else {
            continue;
        };
        let memory_type = memory_type_name(structure.formatted_byte(0x12).unwrap_or_default());
        let manufacturer = structure
            .formatted_byte(0x17)
            .and_then(|index| structure.string_at(index));

        memory.push(MemoryInfo {
            memory_type,
            capacity,
            clock_speed,
            vendor: manufacturer.clone(),
            manufacturer,
            serial_number: structure
                .formatted_byte(0x18)
                .and_then(|index| structure.string_at(index)),
            part_number: structure
                .formatted_byte(0x1a)
                .and_then(|index| structure.string_at(index)),
            device_locator: structure
                .formatted_byte(0x10)
                .and_then(|index| structure.string_at(index)),
            bank_locator: structure
                .formatted_byte(0x11)
                .and_then(|index| structure.string_at(index)),
        });
    }

    Ok(memory)
}

#[cfg(target_os = "windows")]
fn memory_device_size_mb(structure: &super::smbios::SmbiosStructure) -> Option<u32> {
    match structure.formatted_word(0x0c)? {
        0 | 0xffff => None,
        0x7fff => {
            let size = structure.formatted_dword(0x1c)?;
            if size == 0 { None } else { Some(size) }
        }
        size if size & 0x8000 != 0 => {
            let kb_size = (size & 0x7fff) as u32;
            if kb_size == 0 {
                None
            } else {
                Some(kb_size / 1024)
            }
        }
        size => Some(size as u32),
    }
}

#[cfg(target_os = "windows")]
fn memory_device_speed_mts(structure: &super::smbios::SmbiosStructure) -> Option<u16> {
    let speed = structure.formatted_word(0x15)?;

    match speed {
        0 => None,
        0xffff => {
            let extended_speed = structure.formatted_dword(0x54)?;
            u16::try_from(extended_speed).ok().filter(|value| *value > 0)
        }
        value => Some(value),
    }
}

#[cfg(target_os = "windows")]
fn memory_type_name(code: u8) -> String {
    let name = match code {
        0x01 => "Other",
        0x02 => "Unknown",
        0x03 => "DRAM",
        0x04 => "EDRAM",
        0x05 => "VRAM",
        0x06 => "SRAM",
        0x07 => "RAM",
        0x08 => "ROM",
        0x09 => "FLASH",
        0x0f => "SDRAM",
        0x12 => "DDR",
        0x13 => "DDR2",
        0x18 => "DDR3",
        0x1a => "DDR4",
        0x1b => "LPDDR",
        0x1c => "LPDDR2",
        0x1d => "LPDDR3",
        0x1e => "LPDDR4",
        0x1f => "Logical Non-Volatile Device",
        0x20 => "HBM",
        0x21 => "HBM2",
        0x22 => "DDR5",
        0x23 => "LPDDR5",
        _ => return format!("Unknown (0x{code:02X})"),
    };

    name.to_string()
}

#[cfg(test)]
#[path = "memory/tests.rs"]
mod tests;
