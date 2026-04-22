use crate::hw::smbios::{join_non_empty, parse_smbios_structures, smbios_table_bytes};
use super::{BiosInfo, HwResult};

pub fn parse_bios_info_from_smbios(raw_smbios: &[u8]) -> HwResult<BiosInfo> {
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
