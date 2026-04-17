use super::HwResult;

#[cfg(target_os = "windows")]
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

#[cfg(target_os = "windows")]
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

#[cfg(target_os = "windows")]
#[derive(Debug)]
pub struct SmbiosStructure {
    pub structure_type: u8,
    formatted: Vec<u8>,
    strings: Vec<String>,
}

#[cfg(target_os = "windows")]
impl SmbiosStructure {
    pub fn formatted_byte(&self, offset: usize) -> Option<u8> {
        self.formatted.get(offset).copied()
    }

    pub fn formatted_word(&self, offset: usize) -> Option<u16> {
        let bytes = self.formatted.get(offset..offset + 2)?;
        Some(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub fn formatted_dword(&self, offset: usize) -> Option<u32> {
        let bytes = self.formatted.get(offset..offset + 4)?;
        Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn string_at(&self, index: u8) -> Option<String> {
        if index == 0 {
            return None;
        }

        self.strings.get(index as usize - 1).cloned()
    }

    pub fn uuid(&self) -> Option<String> {
        let bytes = self.formatted.get(8..24)?;
        if bytes.iter().all(|byte| *byte == 0) || bytes.iter().all(|byte| *byte == 0xff) {
            return None;
        }

        Some(format!(
            "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            u16::from_le_bytes([bytes[4], bytes[5]]),
            u16::from_le_bytes([bytes[6], bytes[7]]),
            bytes[8],
            bytes[9],
            bytes[10],
            bytes[11],
            bytes[12],
            bytes[13],
            bytes[14],
            bytes[15],
        ))
    }
}

#[cfg(target_os = "windows")]
pub fn parse_smbios_structures(table: &[u8]) -> Vec<SmbiosStructure> {
    let mut structures = Vec::new();
    let mut offset = 0;

    while offset + 4 <= table.len() {
        let structure_type = table[offset];
        let length = table[offset + 1] as usize;

        if structure_type == 127 || length < 4 || offset + length > table.len() {
            break;
        }

        let strings_start = offset + length;
        let Some(strings_end) = find_structure_end(table, strings_start) else {
            break;
        };

        structures.push(SmbiosStructure {
            structure_type,
            formatted: table[offset..offset + length].to_vec(),
            strings: parse_smbios_strings(&table[strings_start..strings_end]),
        });

        offset = strings_end + 2;
    }

    structures
}

#[cfg(target_os = "windows")]
fn find_structure_end(table: &[u8], start: usize) -> Option<usize> {
    if start >= table.len() {
        return None;
    }

    table[start..]
        .windows(2)
        .position(|window| window == [0, 0])
        .map(|position| start + position)
}

#[cfg(target_os = "windows")]
fn parse_smbios_strings(bytes: &[u8]) -> Vec<String> {
    bytes
        .split(|byte| *byte == 0)
        .filter(|value| !value.is_empty())
        .map(|value| String::from_utf8_lossy(value).trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

#[cfg(target_os = "windows")]
pub fn join_non_empty(values: &[String]) -> String {
    values
        .iter()
        .filter(|value| !value.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests;
