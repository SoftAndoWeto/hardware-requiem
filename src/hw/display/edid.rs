//! Pure EDID parsing, no platform dependencies.
//!
//! EDID (Extended Display Identification Data) is a 128-byte binary structure
//! that monitors expose to the host. The base block contains manufacturer info,
//! physical dimensions, and up to four 18-byte descriptor blocks. Descriptor
//! tag `0xFC` holds the monitor name as ASCII.

/// Parsed fields from the EDID base block (first 128 bytes).
#[derive(Debug, Default)]
pub(super) struct EdidInfo {
    pub vendor: Option<String>,
    pub manufacturer_id: Option<String>,
    pub product_code: Option<u16>,
    pub serial_number: Option<u32>,
    pub manufacture_week: Option<u8>,
    pub manufacture_year: Option<u16>,
    pub width_cm: Option<u8>,
    pub height_cm: Option<u8>,
    pub diagonal_inches: Option<f32>,
}

/// Scans the four 18-byte descriptor blocks (bytes 54–125) for the monitor
/// name descriptor (tag `0xFC`) and returns its content as a trimmed string.
pub(super) fn edid_display_name(edid: &[u8]) -> Option<String> {
    for descriptor in edid.get(54..126)?.chunks_exact(18) {
        if descriptor[0..3] == [0, 0, 0] && descriptor[3] == 0xfc {
            let name = descriptor[5..18]
                .iter()
                .copied()
                .take_while(|byte| *byte != 0x0a && *byte != 0x00)
                .filter(|byte| byte.is_ascii_graphic() || *byte == b' ')
                .map(char::from)
                .collect::<String>()
                .trim()
                .to_string();

            if !name.is_empty() {
                return Some(name);
            }
        }
    }

    None
}

/// Parses manufacturer info and physical properties from the EDID base block.
/// Returns a default `EdidInfo` if `edid` is shorter than 128 bytes.
pub(super) fn parse_edid_info(edid: &[u8]) -> EdidInfo {
    let Some(header) = edid.get(0..128) else {
        return EdidInfo::default();
    };

    let manufacturer_id = edid_manufacturer_id(header);
    let product_code = u16::from_le_bytes([header[10], header[11]]);
    let serial_number = u32::from_le_bytes([header[12], header[13], header[14], header[15]]);
    let manufacture_week = match header[16] {
        0 | 0xff => None,
        week => Some(week),
    };
    let manufacture_year = match header[17] {
        0 => None,
        year_offset => Some(1990 + year_offset as u16),
    };
    let width_cm = non_zero_byte(header[21]);
    let height_cm = non_zero_byte(header[22]);

    EdidInfo {
        vendor: manufacturer_id
            .as_deref()
            .and_then(edid_vendor_name)
            .map(str::to_string)
            .or_else(|| manufacturer_id.clone()),
        manufacturer_id,
        product_code: non_zero_u16(product_code),
        serial_number: non_zero_u32(serial_number),
        manufacture_week,
        manufacture_year,
        width_cm,
        height_cm,
        diagonal_inches: diagonal_inches(width_cm, height_cm),
    }
}

/// Decodes the 3-letter manufacturer ID from EDID bytes 8–9.
///
/// The two bytes pack three 5-bit values (A=1…Z=26) into a big-endian `u16`.
/// Returns `None` if any character falls outside the valid A–Z range.
pub(super) fn edid_manufacturer_id(edid: &[u8]) -> Option<String> {
    let value = u16::from_be_bytes([*edid.get(8)?, *edid.get(9)?]);
    let mut manufacturer = String::with_capacity(3);

    for shift in [10, 5, 0] {
        let character = ((value >> shift) & 0x1f) as u8;
        if !(1..=26).contains(&character) {
            return None;
        }
        manufacturer.push((b'A' + character - 1) as char);
    }

    Some(manufacturer)
}

/// Maps known 3-letter manufacturer IDs to full brand names.
/// Returns `None` for IDs not in the table.
pub(super) fn edid_vendor_name(manufacturer_id: &str) -> Option<&'static str> {
    match manufacturer_id {
        "SAM" => Some("Samsung"),
        "DEL" => Some("Dell"),
        "GSM" => Some("LG"),
        "ACR" => Some("Acer"),
        "AOC" => Some("AOC"),
        "APP" => Some("Apple"),
        "AUS" => Some("ASUS"),
        "BNQ" => Some("BenQ"),
        "HWP" => Some("HP"),
        "LEN" => Some("Lenovo"),
        "MSI" => Some("MSI"),
        "SNY" => Some("Sony"),
        "VSC" => Some("ViewSonic"),
        _ => None,
    }
}

pub(super) fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut output = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }

    output
}

fn non_zero_byte(value: u8) -> Option<u8> {
    if value == 0 { None } else { Some(value) }
}

fn non_zero_u16(value: u16) -> Option<u16> {
    if value == 0 { None } else { Some(value) }
}

fn non_zero_u32(value: u32) -> Option<u32> {
    if value == 0 { None } else { Some(value) }
}

fn diagonal_inches(width_cm: Option<u8>, height_cm: Option<u8>) -> Option<f32> {
    let width_cm = width_cm? as f32;
    let height_cm = height_cm? as f32;
    let diagonal = (width_cm.powi(2) + height_cm.powi(2)).sqrt() / 2.54;

    Some((diagonal * 10.0).round() / 10.0)
}
