use serde::{Deserialize, Serialize};

use super::{
    smbios::{parse_smbios_structures, read_raw_smbios_table, smbios_table_bytes},
    HwResult,
};

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
#[cfg(target_os = "windows")]
pub fn get_motherboard_info() -> HwResult<MotherboardInfo> {
    let smbios = read_raw_smbios_table()?;
    parse_motherboard_info_from_smbios(&smbios)
}

#[cfg(not(target_os = "windows"))]
pub fn get_motherboard_info() -> HwResult<MotherboardInfo> {
    Err("motherboard collection is only implemented on Windows".to_string())
}

#[cfg(target_os = "windows")]
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

#[cfg(target_os = "windows")]
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

#[cfg(target_os = "windows")]
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

#[cfg(target_os = "windows")]
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
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn parses_motherboard_from_smbios_baseboard() {
        let mut table = Vec::new();
        table.extend_from_slice(&[
            0x02, 0x0F, 0x01, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x19, 0x06, 0x00, 0x00, 0x0A,
            0x00,
        ]);
        table.extend_from_slice(b"ASUSTeK COMPUTER INC.\0");
        table.extend_from_slice(b"ROG STRIX Z790-E GAMING WIFI\0");
        table.extend_from_slice(b"Rev 1.xx\0");
        table.extend_from_slice(b"1234567890\0");
        table.extend_from_slice(b"Default string\0");
        table.extend_from_slice(b"To Be Filled By O.E.M.\0\0");
        table.extend_from_slice(&[
            0x10, 0x0F, 0x02, 0x00, 0x03, 0x03, 0x03, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0x06,
            0x00,
        ]);
        table.extend_from_slice(b"\0\0");
        table.extend_from_slice(&[
            0x11, 0x15, 0x03, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x00, 0x40,
            0x09, 0x00, 0x01, 0x00, 0x18, 0x00, 0x15,
        ]);
        table.extend_from_slice(b"DIMM_A1\0BANK 0\0\0");
        table.extend_from_slice(&[
            0x11, 0x15, 0x04, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x02, 0x00, 0x00, 0x00,
            0x09, 0x00, 0x01, 0x00, 0x18, 0x00, 0x15,
        ]);
        table.extend_from_slice(b"DIMM_A2\0BANK 1\0\0");
        table.extend_from_slice(&[
            0x04, 0x1A, 0x05, 0x00, 0x01, 0x03, 0xB3, 0xFE, 0xFF, 0xFB, 0x8B, 0x17, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xE8, 0x03, 0x10, 0x0E, 0x41, 0x00, 0x41, 0x00,
        ]);
        table.extend_from_slice(b"LGA1700\0Intel\0\0");

        let mut raw_smbios = vec![0, 3, 4, 0];
        raw_smbios.extend_from_slice(&(table.len() as u32).to_le_bytes());
        raw_smbios.extend_from_slice(&table);

        let motherboard = parse_motherboard_info_from_smbios(&raw_smbios).unwrap();

        assert_eq!(motherboard.vendor, "ASUSTeK COMPUTER INC.");
        assert_eq!(motherboard.manufacturer, "ASUSTeK COMPUTER INC.");
        assert_eq!(motherboard.serial_number, "1234567890");
        assert_eq!(
            motherboard.product.as_deref(),
            Some("ROG STRIX Z790-E GAMING WIFI")
        );
        assert_eq!(motherboard.version.as_deref(), Some("Rev 1.xx"));
        assert_eq!(motherboard.asset_tag.as_deref(), Some("Default string"));
        assert_eq!(
            motherboard.location_in_chassis.as_deref(),
            Some("To Be Filled By O.E.M.")
        );
        assert_eq!(motherboard.board_type.as_deref(), Some("Motherboard"));
        assert_eq!(
            motherboard.feature_flags,
            vec![
                "Hosting Board".to_string(),
                "Replaceable".to_string(),
                "Hot Swappable".to_string()
            ]
        );
        assert_eq!(motherboard.memory_slot_count, Some(6));
        assert_eq!(motherboard.occupied_memory_slot_count, Some(1));
        assert_eq!(motherboard.processor_socket.as_deref(), Some("LGA1700"));
    }
}
