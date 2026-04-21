use super::HwResult;

pub fn read_raw_smbios_table() -> HwResult<Vec<u8>> {
    use windows::Win32::System::SystemInformation::{GetSystemFirmwareTable, RSMB};

    let size = unsafe { GetSystemFirmwareTable(RSMB, 0, None) };
    if size == 0 {
        return Err("cannot get SMBIOS firmware table size".to_string());
    }

    let mut buffer = vec![0; size as usize];
    let written = unsafe { GetSystemFirmwareTable(RSMB, 0, Some(&mut buffer)) };
    if written == 0 {
        return Err("cannot read SMBIOS firmware table".to_string());
    }

    buffer.truncate(written as usize);
    Ok(buffer)
}

pub fn smbios_table_bytes(raw_smbios: &[u8]) -> HwResult<&[u8]> {
    if raw_smbios.len() < 8 {
        return Err("SMBIOS firmware table is too small".to_string());
    }

    let table_len =
        u32::from_le_bytes([raw_smbios[4], raw_smbios[5], raw_smbios[6], raw_smbios[7]]) as usize;
    let table_start = 8;
    let table_end = table_start + table_len;

    raw_smbios
        .get(table_start..table_end)
        .or_else(|| raw_smbios.get(table_start..))
        .ok_or_else(|| "SMBIOS table data is missing".to_string())
}
