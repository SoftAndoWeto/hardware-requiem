use super::*;

#[test]
#[cfg(target_os = "windows")]
fn extracts_table_bytes_from_raw_smbios_data() {
    let table = [1, 2, 3, 4, 5];
    let mut raw_smbios = vec![0, 3, 4, 0];
    raw_smbios.extend_from_slice(&(table.len() as u32).to_le_bytes());
    raw_smbios.extend_from_slice(&table);

    assert_eq!(smbios_table_bytes(&raw_smbios).unwrap(), table);
}

#[test]
#[cfg(target_os = "windows")]
fn rejects_too_short_raw_smbios_data() {
    let error = smbios_table_bytes(&[0, 1, 2, 3]).unwrap_err();

    assert_eq!(error, "SMBIOS firmware table is too small");
}

#[test]
#[cfg(target_os = "windows")]
fn falls_back_to_remaining_table_bytes_when_declared_length_is_too_large() {
    let table = [9, 8, 7];
    let mut raw_smbios = vec![0, 3, 4, 0];
    raw_smbios.extend_from_slice(&10u32.to_le_bytes());
    raw_smbios.extend_from_slice(&table);

    assert_eq!(smbios_table_bytes(&raw_smbios).unwrap(), table);
}

#[test]
#[cfg(target_os = "windows")]
fn parses_single_smbios_structure_with_strings() {
    let table = [
        0x02, 0x08, 0x34, 0x12, 0x01, 0x02, 0x78, 0x56, b'M', b'S', b'I', 0x00, b'Z', b'7', b'9',
        b'0', 0x00, 0x00,
    ];

    let structures = parse_smbios_structures(&table);

    assert_eq!(structures.len(), 1);
    assert_eq!(structures[0].structure_type, 2);
    assert_eq!(structures[0].formatted_byte(4), Some(1));
    assert_eq!(structures[0].formatted_word(2), Some(0x1234));
    assert_eq!(structures[0].string_at(1), Some("MSI".to_string()));
    assert_eq!(structures[0].string_at(2), Some("Z790".to_string()));
    assert_eq!(structures[0].string_at(0), None);
    assert_eq!(structures[0].string_at(3), None);
}

#[test]
#[cfg(target_os = "windows")]
fn parses_multiple_structures_and_stops_at_end_of_table_marker() {
    let table = [
        0x00, 0x05, 0x00, 0x00, 0x01, b'A', 0x00, 0x00, 0x01, 0x05, 0x01, 0x00, 0x01, b'B', 0x00,
        0x00, 0x7f, 0x04, 0x02, 0x00, 0x00, 0x00,
    ];

    let structures = parse_smbios_structures(&table);

    assert_eq!(structures.len(), 2);
    assert_eq!(structures[0].structure_type, 0);
    assert_eq!(structures[1].structure_type, 1);
}

#[test]
#[cfg(target_os = "windows")]
fn stops_on_invalid_structure_length() {
    let table = [0x02, 0x03, 0x00, 0x00, 0x00, 0x00];

    assert!(parse_smbios_structures(&table).is_empty());
}

#[test]
#[cfg(target_os = "windows")]
fn stops_on_missing_string_terminator() {
    let table = [0x02, 0x04, 0x00, 0x00, b'A', 0x00];

    assert!(parse_smbios_structures(&table).is_empty());
}

#[test]
#[cfg(target_os = "windows")]
fn reads_formatted_values() {
    let table = [0x02, 0x08, 0x00, 0x00, 0x78, 0x56, 0x34, 0x12, 0x00, 0x00];
    let structures = parse_smbios_structures(&table);
    let structure = &structures[0];

    assert_eq!(structure.formatted_byte(4), Some(0x78));
    assert_eq!(structure.formatted_word(4), Some(0x5678));
    assert_eq!(structure.formatted_dword(4), Some(0x12345678));
    assert_eq!(structure.formatted_dword(5), None);
}

#[test]
#[cfg(target_os = "windows")]
fn formats_smbios_uuid() {
    let table = [
        0x01, 0x18, 0x00, 0x00, 0, 0, 0, 0, 0x67, 0x45, 0x23, 0x01, 0xab, 0x89, 0xef, 0xcd, 0x01,
        0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x00, 0x00,
    ];
    let structures = parse_smbios_structures(&table);

    assert_eq!(
        structures[0].uuid(),
        Some("01234567-89ab-cdef-0123-456789abcdef".to_string())
    );
}

#[test]
#[cfg(target_os = "windows")]
fn skips_empty_or_unset_uuid() {
    let mut zero_uuid_table = [
        0x01, 0x18, 0x00, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let structures = parse_smbios_structures(&zero_uuid_table);
    assert_eq!(structures[0].uuid(), None);

    zero_uuid_table[8..24].fill(0xff);
    let structures = parse_smbios_structures(&zero_uuid_table);
    assert_eq!(structures[0].uuid(), None);
}

#[test]
#[cfg(target_os = "windows")]
fn joins_only_non_empty_values() {
    assert_eq!(
        join_non_empty(&["A".to_string(), String::new(), "B".to_string()]),
        "A B"
    );
    assert_eq!(join_non_empty(&[String::new(), String::new()]), "");
}
