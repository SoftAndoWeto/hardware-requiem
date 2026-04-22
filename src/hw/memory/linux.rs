use crate::hw::smbios::{read_raw_smbios_table, smbios_table_bytes};

use super::parser::{parse_memory_devices, MemoryInfo};
use super::HwResult;

pub fn get_memory_info() -> HwResult<Vec<MemoryInfo>> {
    match read_raw_smbios_table().and_then(|raw| {
        let table = smbios_table_bytes(&raw)?;
        Ok(parse_memory_devices(table))
    }) {
        Ok(devices) if !devices.is_empty() => Ok(devices),
        _ => read_meminfo_fallback(),
    }
}

fn read_meminfo_fallback() -> HwResult<Vec<MemoryInfo>> {
    let content = std::fs::read_to_string("/proc/meminfo")
        .map_err(|e| format!("cannot read /proc/meminfo: {e}"))?;

    let total_kb = content
        .lines()
        .find_map(|line| {
            let rest = line.strip_prefix("MemTotal:")?;
            rest.split_whitespace().next()?.parse::<u64>().ok()
        })
        .ok_or("MemTotal not found in /proc/meminfo")?;

    let capacity_mb = (total_kb / 1024).min(u16::MAX as u64) as u16;

    Ok(vec![MemoryInfo {
        memory_type: "Unknown".to_string(),
        capacity: capacity_mb,
        clock_speed: 0,
        vendor: None,
        manufacturer: None,
        serial_number: None,
        part_number: None,
        device_locator: None,
        bank_locator: None,
    }])
}
