use serde::{Deserialize, Serialize};

use crate::hw::smbios::{parse_smbios_structures, SmbiosStructure};

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

pub fn parse_memory_devices(table: &[u8]) -> Vec<MemoryInfo> {
    let mut memory = Vec::new();

    for structure in parse_smbios_structures(table)
        .iter()
        .filter(|s| s.structure_type == 17)
    {
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

    memory
}

fn memory_device_size_mb(structure: &SmbiosStructure) -> Option<u32> {
    match structure.formatted_word(0x0c)? {
        0 | 0xffff => None,
        0x7fff => {
            let size = structure.formatted_dword(0x1c)?;
            if size == 0 { None } else { Some(size) }
        }
        size if size & 0x8000 != 0 => {
            let kb_size = (size & 0x7fff) as u32;
            if kb_size == 0 { None } else { Some(kb_size / 1024) }
        }
        size => Some(size as u32),
    }
}

fn memory_device_speed_mts(structure: &SmbiosStructure) -> Option<u16> {
    match structure.formatted_word(0x15)? {
        0 => None,
        0xffff => {
            let extended = structure.formatted_dword(0x54)?;
            u16::try_from(extended).ok().filter(|v| *v > 0)
        }
        value => Some(value),
    }
}

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
