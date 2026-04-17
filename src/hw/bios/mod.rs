use serde::{Deserialize, Serialize};

use super::{
    smbios::{join_non_empty, parse_smbios_structures, read_raw_smbios_table, smbios_table_bytes},
    HwResult,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct BiosInfo {
    pub uuid: String,
    pub manufacturer: String,
    pub name: String,
}

#[cfg(target_os = "windows")]
pub fn get_bios_info() -> HwResult<BiosInfo> {
    let smbios = read_raw_smbios_table()?;
    parse_bios_info_from_smbios(&smbios)
}

#[cfg(not(target_os = "windows"))]
pub fn get_bios_info() -> HwResult<BiosInfo> {
    Err("BIOS collection is only implemented on Windows".to_string())
}

#[cfg(target_os = "windows")]
fn parse_bios_info_from_smbios(raw_smbios: &[u8]) -> HwResult<BiosInfo> {
    let table = smbios_table_bytes(raw_smbios)?;
    let structures = parse_smbios_structures(table);

    let bios = structures
        .iter()
        .find(|structure| structure.structure_type == 0)
        .ok_or_else(|| "SMBIOS type 0 BIOS Information was not found".to_string())?;

    let vendor = bios
        .string_at(bios.formatted_byte(4).unwrap_or_default())
        .ok_or_else(|| "SMBIOS BIOS vendor is missing".to_string())?;
    let version = bios
        .string_at(bios.formatted_byte(5).unwrap_or_default())
        .unwrap_or_default();
    let release_date = bios
        .string_at(bios.formatted_byte(8).unwrap_or_default())
        .unwrap_or_default();
    let uuid = structures
        .iter()
        .find(|structure| structure.structure_type == 1)
        .and_then(|structure| structure.uuid())
        .unwrap_or_default();

    Ok(BiosInfo {
        uuid,
        manufacturer: vendor,
        name: join_non_empty(&[version, release_date]),
    })
}

#[cfg(test)]
mod tests;
