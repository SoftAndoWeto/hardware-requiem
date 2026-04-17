use super::*;

#[test]
#[cfg(target_os = "windows")]
fn parses_motherboard_from_smbios_baseboard() {
    let mut table = Vec::new();
    table.extend_from_slice(&[
        0x02, 0x0F, 0x01, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x19, 0x06, 0x00, 0x00, 0x0A, 0x00,
    ]);
    table.extend_from_slice(b"ASUSTeK COMPUTER INC.\0");
    table.extend_from_slice(b"ROG STRIX Z790-E GAMING WIFI\0");
    table.extend_from_slice(b"Rev 1.xx\0");
    table.extend_from_slice(b"1234567890\0");
    table.extend_from_slice(b"Default string\0");
    table.extend_from_slice(b"To Be Filled By O.E.M.\0\0");
    table.extend_from_slice(&[
        0x10, 0x0F, 0x02, 0x00, 0x03, 0x03, 0x03, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0x06, 0x00,
    ]);
    table.extend_from_slice(b"\0\0");
    table.extend_from_slice(&[
        0x11, 0x15, 0x03, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x00, 0x40, 0x09,
        0x00, 0x01, 0x00, 0x18, 0x00, 0x15,
    ]);
    table.extend_from_slice(b"DIMM_A1\0BANK 0\0\0");
    table.extend_from_slice(&[
        0x11, 0x15, 0x04, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x02, 0x00, 0x00, 0x00, 0x09,
        0x00, 0x01, 0x00, 0x18, 0x00, 0x15,
    ]);
    table.extend_from_slice(b"DIMM_A2\0BANK 1\0\0");
    table.extend_from_slice(&[
        0x04, 0x1A, 0x05, 0x00, 0x01, 0x03, 0xB3, 0xFE, 0xFF, 0xFB, 0x8B, 0x17, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0xE8, 0x03, 0x10, 0x0E, 0x41, 0x00, 0x41, 0x00,
    ]);
    table.extend_from_slice(b"LGA1700\0Intel\0\0");

    let mut raw_smbios = vec![0, 3, 4, 0];
    raw_smbios.extend_from_slice(&(table.len() as u32).to_le_bytes());
    raw_smbios.extend_from_slice(&table);

    let motherboard = parse_motherboard_info_from_smbios(&raw_smbios).unwrap();

    assert_eq!(motherboard.vendor, "ASUSTeK COMPUTER INC.");
    assert_eq!(motherboard.manufacturer, "ASUSTeK COMPUTER INC.");
    assert_eq!(motherboard.serial_number, "1234567890");
    assert_eq!(
        motherboard.product.as_deref(),
        Some("ROG STRIX Z790-E GAMING WIFI")
    );
    assert_eq!(motherboard.version.as_deref(), Some("Rev 1.xx"));
    assert_eq!(motherboard.asset_tag.as_deref(), Some("Default string"));
    assert_eq!(
        motherboard.location_in_chassis.as_deref(),
        Some("To Be Filled By O.E.M.")
    );
    assert_eq!(motherboard.board_type.as_deref(), Some("Motherboard"));
    assert_eq!(
        motherboard.feature_flags,
        vec![
            "Hosting Board".to_string(),
            "Replaceable".to_string(),
            "Hot Swappable".to_string()
        ]
    );
    assert_eq!(motherboard.memory_slot_count, Some(6));
    assert_eq!(motherboard.occupied_memory_slot_count, Some(1));
    assert_eq!(motherboard.processor_socket.as_deref(), Some("LGA1700"));
}
