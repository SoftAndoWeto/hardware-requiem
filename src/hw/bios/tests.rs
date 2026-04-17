use super::*;

#[test]
#[cfg(target_os = "windows")]
fn parses_bios_and_system_uuid_from_smbios() {
    let mut table = Vec::new();
    table.extend_from_slice(&[0x00, 0x09, 0x00, 0x00, 0x01, 0x02, 0x00, 0x00, 0x03]);
    table.extend_from_slice(b"American Megatrends International, LLC.\0");
    table.extend_from_slice(b"ALASKA - 1072009\0");
    table.extend_from_slice(b"08/08/2024\0\0");
    table.extend_from_slice(&[
        0x01, 0x19, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x67, 0x45, 0x23, 0x01, 0xab, 0x89, 0xef,
        0xcd, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x00,
    ]);
    table.extend_from_slice(b"\0\0");

    let mut raw_smbios = vec![0, 3, 4, 0];
    raw_smbios.extend_from_slice(&(table.len() as u32).to_le_bytes());
    raw_smbios.extend_from_slice(&table);

    let bios = parse_bios_info_from_smbios(&raw_smbios).unwrap();

    assert_eq!(bios.manufacturer, "American Megatrends International, LLC.");
    assert_eq!(bios.name, "ALASKA - 1072009 08/08/2024");
    assert_eq!(bios.uuid, "01234567-89ab-cdef-0123-456789abcdef");
}
