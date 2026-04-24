use crate::hw::smbios::read_raw_smbios_table;
use super::{parser::parse_motherboard_info_from_smbios, HwResult, MotherboardInfo};

pub fn get_motherboard_info() -> HwResult<MotherboardInfo> {
    let smbios = read_raw_smbios_table()?;
    parse_motherboard_info_from_smbios(&smbios)
}
