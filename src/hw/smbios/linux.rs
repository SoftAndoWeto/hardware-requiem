use super::HwResult;

pub fn read_raw_smbios_table() -> HwResult<Vec<u8>> {
    std::fs::read("/sys/firmware/dmi/tables/DMI")
        .map_err(|error| format!("cannot read SMBIOS table: {error}"))
}

pub fn smbios_table_bytes(raw_smbios: &[u8]) -> HwResult<&[u8]> {
    Ok(raw_smbios)
}
