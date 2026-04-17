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
    pub manufacturer: String,
    #[serde(rename = "serialNumber")]
    pub serial_number: String,
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

    Ok(MotherboardInfo {
        manufacturer,
        serial_number,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn parses_motherboard_from_smbios_baseboard() {
        let mut table = Vec::new();
        table.extend_from_slice(&[0x02, 0x08, 0x01, 0x00, 0x01, 0x02, 0x03, 0x04]);
        table.extend_from_slice(b"ASUSTeK COMPUTER INC.\0");
        table.extend_from_slice(b"ROG STRIX Z790-E GAMING WIFI\0");
        table.extend_from_slice(b"Rev 1.xx\0");
        table.extend_from_slice(b"1234567890\0\0");

        let mut raw_smbios = vec![0, 3, 4, 0];
        raw_smbios.extend_from_slice(&(table.len() as u32).to_le_bytes());
        raw_smbios.extend_from_slice(&table);

        let motherboard = parse_motherboard_info_from_smbios(&raw_smbios).unwrap();

        assert_eq!(motherboard.manufacturer, "ASUSTeK COMPUTER INC.");
        assert_eq!(motherboard.serial_number, "1234567890");
    }
}
