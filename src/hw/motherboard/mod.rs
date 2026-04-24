use serde::{Deserialize, Serialize};

use super::HwResult;

#[cfg(any(windows, target_os = "linux"))]
use super::smbios::{parse_smbios_structures, read_raw_smbios_table, smbios_table_bytes};

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

/// Retrieves motherboard information from SMBIOS type 2 Baseboard Information.
#[cfg(any(windows, target_os = "linux"))]
pub fn get_motherboard_info() -> HwResult<MotherboardInfo> {
    let smbios = read_raw_smbios_table()?;
    parse_motherboard_info_from_smbios(&smbios)
}

#[cfg(not(any(windows, target_os = "linux")))]
pub fn get_motherboard_info() -> HwResult<MotherboardInfo> {
    Err("motherboard collection is not implemented for this platform".to_string())
}

#[cfg(any(windows, target_os = "linux"))]
fn parse_motherboard_info_from_smbios(raw_smbios: &[u8]) -> HwResult<MotherboardInfo> {
    let table = smbios_table_bytes(raw_smbios)?;
    let structures = parse_smbios_structures(table);

    let baseboard = structures
        .iter()
        .find(|structure| structure.structure_type == 2)
        .ok_or_else(|| "SMBIOS type 2 Baseboard Information was not found".to_string())?;

    let manufacturer = baseboard
        .string_at(baseboard.formatted_byte(4).unwrap_or_default())
        .ok_or_else(|| "SMBIOS baseboard manufacturer is missing".to_string())?;
    let serial_number = baseboard
        .string_at(baseboard.formatted_byte(7).unwrap_or_default())
        .unwrap_or_default();
    let product = baseboard.string_at(baseboard.formatted_byte(5).unwrap_or_default());
    let version = baseboard.string_at(baseboard.formatted_byte(6).unwrap_or_default());
    let asset_tag = baseboard.string_at(baseboard.formatted_byte(8).unwrap_or_default());
    let location_in_chassis = baseboard.string_at(baseboard.formatted_byte(10).unwrap_or_default());
    let board_type = baseboard
        .formatted_byte(13)
        .and_then(baseboard_type_description);
    let feature_flags = baseboard
        .formatted_byte(9)
        .map(baseboard_feature_flags)
        .unwrap_or_default();
    let memory_slot_count = structures
        .iter()
        .find(|structure| structure.structure_type == 16)
        .and_then(|structure| structure.formatted_word(13))
        .filter(|count| *count > 0);
    let occupied_memory_slot_count = Some(
        structures
            .iter()
            .filter(|structure| structure.structure_type == 17)
            .filter(|structure| memory_device_is_installed(structure))
            .count() as u16,
    )
    .filter(|count| *count > 0);
    let processor_socket = structures
        .iter()
        .find(|structure| structure.structure_type == 4)
        .and_then(|structure| structure.string_at(structure.formatted_byte(4).unwrap_or_default()));

    Ok(MotherboardInfo {
        vendor: manufacturer.clone(),
        manufacturer,
        product,
        version,
        serial_number,
        asset_tag,
        location_in_chassis,
        board_type,
        feature_flags,
        memory_slot_count,
        occupied_memory_slot_count,
        processor_socket,
    })
}

#[cfg(any(windows, target_os = "linux"))]
fn baseboard_type_description(value: u8) -> Option<String> {
    let description = match value {
        0x01 => "Unknown",
        0x02 => "Other",
        0x03 => "Server Blade",
        0x04 => "Connectivity Switch",
        0x05 => "System Management Module",
        0x06 => "Processor Module",
        0x07 => "I/O Module",
        0x08 => "Memory Module",
        0x09 => "Daughter Board",
        0x0A => "Motherboard",
        0x0B => "Processor/Memory Module",
        0x0C => "Processor/IO Module",
        0x0D => "Interconnect Board",
        _ => return None,
    };

    Some(description.to_string())
}

#[cfg(any(windows, target_os = "linux"))]
fn baseboard_feature_flags(value: u8) -> Vec<String> {
    [
        (0x01, "Hosting Board"),
        (0x02, "Requires Daughter Board"),
        (0x04, "Removable"),
        (0x08, "Replaceable"),
        (0x10, "Hot Swappable"),
    ]
    .into_iter()
    .filter_map(|(flag, description)| {
        if value & flag != 0 {
            Some(description.to_string())
        } else {
            None
        }
    })
    .collect()
}

#[cfg(any(windows, target_os = "linux"))]
fn memory_device_is_installed(structure: &super::smbios::SmbiosStructure) -> bool {
    match structure.formatted_word(12) {
        Some(0) | Some(0xffff) | None => false,
        Some(0x7fff) => structure
            .formatted_dword(28)
            .map(|extended_size| extended_size > 0)
            .unwrap_or(false),
        Some(_) => true,
    }
}

#[cfg(test)]
mod tests;
