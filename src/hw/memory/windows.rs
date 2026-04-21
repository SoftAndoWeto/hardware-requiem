use crate::hw::smbios::{read_raw_smbios_table, smbios_table_bytes};

use super::parser::{parse_memory_devices, MemoryInfo};
use super::HwResult;

pub fn get_memory_info() -> HwResult<Vec<MemoryInfo>> {
    let raw = read_raw_smbios_table()?;
    let table = smbios_table_bytes(&raw)?;
    Ok(parse_memory_devices(table))
}
