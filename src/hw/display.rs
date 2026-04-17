use serde::{Deserialize, Serialize};

use super::HwResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub name: String,
    pub vendor: Option<String>,
    #[serde(rename = "hardwareId")]
    pub hardware_id: Option<String>,
    #[serde(rename = "deviceId")]
    pub device_id: Option<String>,
    #[serde(rename = "adapterName")]
    pub adapter_name: Option<String>,
    #[serde(rename = "adapterDeviceName")]
    pub adapter_device_name: Option<String>,
    #[serde(rename = "isPrimary")]
    pub is_primary: Option<bool>,
    #[serde(rename = "currentResolutionWidth")]
    pub current_resolution_width: Option<u32>,
    #[serde(rename = "currentResolutionHeight")]
    pub current_resolution_height: Option<u32>,
    #[serde(rename = "refreshRateHz")]
    pub refresh_rate_hz: Option<u32>,
    #[serde(rename = "bitsPerPixel")]
    pub bits_per_pixel: Option<u32>,
    #[serde(rename = "positionX")]
    pub position_x: Option<i32>,
    #[serde(rename = "positionY")]
    pub position_y: Option<i32>,
    #[serde(rename = "manufacturerId")]
    pub manufacturer_id: Option<String>,
    #[serde(rename = "productCode")]
    pub product_code: Option<u16>,
    #[serde(rename = "serialNumber")]
    pub serial_number: Option<u32>,
    #[serde(rename = "manufactureWeek")]
    pub manufacture_week: Option<u8>,
    #[serde(rename = "manufactureYear")]
    pub manufacture_year: Option<u16>,
    #[serde(rename = "widthCm")]
    pub width_cm: Option<u8>,
    #[serde(rename = "heightCm")]
    pub height_cm: Option<u8>,
    #[serde(rename = "diagonalInches")]
    pub diagonal_inches: Option<f32>,
    pub edid: Option<String>,
}

#[cfg(target_os = "windows")]
pub fn get_display() -> HwResult<Vec<DisplayInfo>> {
    let active_monitors = collect_active_monitors();
    let mut displays = collect_edid_displays(&active_monitors)?;

    if displays.is_empty() {
        displays = active_monitors
            .into_iter()
            .map(DisplayInfo::from_active_monitor)
            .collect();
    }

    Ok(displays)
}

#[cfg(not(target_os = "windows"))]
pub fn get_display() -> HwResult<Vec<DisplayInfo>> {
    Ok(Vec::new())
}

#[cfg(target_os = "windows")]
#[derive(Debug)]
struct EdidDisplay {
    hardware_id: String,
    name: Option<String>,
    edid: Vec<u8>,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone)]
struct ActiveMonitor {
    hardware_id: Option<String>,
    device_id: String,
    name: String,
    adapter_name: String,
    adapter_device_name: String,
    is_primary: bool,
    settings: Option<DisplaySettings>,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone)]
struct DisplaySettings {
    width: u32,
    height: u32,
    refresh_rate_hz: u32,
    bits_per_pixel: u32,
    position_x: i32,
    position_y: i32,
}

#[cfg(target_os = "windows")]
impl DisplayInfo {
    fn from_active_monitor(monitor: ActiveMonitor) -> Self {
        let settings = monitor.settings;

        Self {
            name: monitor.name,
            vendor: None,
            hardware_id: monitor.hardware_id,
            device_id: Some(monitor.device_id),
            adapter_name: Some(monitor.adapter_name),
            adapter_device_name: Some(monitor.adapter_device_name),
            is_primary: Some(monitor.is_primary),
            current_resolution_width: settings.as_ref().map(|settings| settings.width),
            current_resolution_height: settings.as_ref().map(|settings| settings.height),
            refresh_rate_hz: settings.as_ref().map(|settings| settings.refresh_rate_hz),
            bits_per_pixel: settings.as_ref().map(|settings| settings.bits_per_pixel),
            position_x: settings.as_ref().map(|settings| settings.position_x),
            position_y: settings.as_ref().map(|settings| settings.position_y),
            manufacturer_id: None,
            product_code: None,
            serial_number: None,
            manufacture_week: None,
            manufacture_year: None,
            width_cm: None,
            height_cm: None,
            diagonal_inches: None,
            edid: None,
        }
    }
}

#[cfg(target_os = "windows")]
#[derive(Debug, Default)]
struct EdidInfo {
    vendor: Option<String>,
    manufacturer_id: Option<String>,
    product_code: Option<u16>,
    serial_number: Option<u32>,
    manufacture_week: Option<u8>,
    manufacture_year: Option<u16>,
    width_cm: Option<u8>,
    height_cm: Option<u8>,
    diagonal_inches: Option<f32>,
}

#[cfg(target_os = "windows")]
fn collect_active_monitors() -> Vec<ActiveMonitor> {
    use windows::core::PCWSTR;
    use windows::Win32::Graphics::Gdi::{
        EnumDisplayDevicesW, DISPLAY_DEVICEW, DISPLAY_DEVICE_ACTIVE, DISPLAY_DEVICE_PRIMARY_DEVICE,
    };
    use windows::Win32::UI::WindowsAndMessaging::EDD_GET_DEVICE_INTERFACE_NAME;

    let mut monitors = Vec::new();
    let mut adapter_index = 0;

    loop {
        let mut adapter = DISPLAY_DEVICEW {
            cb: size_of::<DISPLAY_DEVICEW>() as u32,
            ..Default::default()
        };

        if !unsafe { EnumDisplayDevicesW(PCWSTR::null(), adapter_index, &mut adapter, 0).as_bool() }
        {
            break;
        }

        let adapter_name = utf16_null_terminated_to_string(&adapter.DeviceName);
        let adapter_device_name = wide_null_terminated(&adapter_name);
        let adapter_display_name = utf16_null_terminated_to_string(&adapter.DeviceString);
        let is_primary = (adapter.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE) != 0;
        let settings = display_settings(&adapter_name);
        let mut monitor_index = 0;

        loop {
            let mut monitor = DISPLAY_DEVICEW {
                cb: size_of::<DISPLAY_DEVICEW>() as u32,
                ..Default::default()
            };

            if !unsafe {
                EnumDisplayDevicesW(
                    PCWSTR(adapter_device_name.as_ptr()),
                    monitor_index,
                    &mut monitor,
                    EDD_GET_DEVICE_INTERFACE_NAME,
                )
                .as_bool()
            } {
                break;
            }

            if (monitor.StateFlags & DISPLAY_DEVICE_ACTIVE) != 0 {
                let device_id = utf16_null_terminated_to_string(&monitor.DeviceID);
                let hardware_id = monitor_hardware_id(&device_id);
                let name = utf16_null_terminated_to_string(&monitor.DeviceString);

                monitors.push(ActiveMonitor {
                    hardware_id,
                    device_id,
                    name,
                    adapter_name: adapter_display_name.clone(),
                    adapter_device_name: adapter_name.clone(),
                    is_primary,
                    settings: settings.clone(),
                });
            }

            monitor_index += 1;
        }

        adapter_index += 1;
    }

    monitors
}

#[cfg(target_os = "windows")]
fn display_settings(adapter_device_name: &str) -> Option<DisplaySettings> {
    use windows::core::PCWSTR;
    use windows::Win32::Graphics::Gdi::{EnumDisplaySettingsW, DEVMODEW, ENUM_CURRENT_SETTINGS};

    let adapter_device_name = wide_null_terminated(adapter_device_name);
    let mut dev_mode = DEVMODEW {
        dmSize: size_of::<DEVMODEW>() as u16,
        ..Default::default()
    };

    if !unsafe {
        EnumDisplaySettingsW(
            PCWSTR(adapter_device_name.as_ptr()),
            ENUM_CURRENT_SETTINGS,
            &mut dev_mode,
        )
        .as_bool()
    } {
        return None;
    }

    let position = unsafe { dev_mode.Anonymous1.Anonymous2.dmPosition };

    Some(DisplaySettings {
        width: dev_mode.dmPelsWidth,
        height: dev_mode.dmPelsHeight,
        refresh_rate_hz: dev_mode.dmDisplayFrequency,
        bits_per_pixel: dev_mode.dmBitsPerPel,
        position_x: position.x,
        position_y: position.y,
    })
}

#[cfg(target_os = "windows")]
fn collect_edid_displays(active_monitors: &[ActiveMonitor]) -> HwResult<Vec<DisplayInfo>> {
    let mut displays = Vec::new();

    for display in enumerate_edid_displays()? {
        let active_monitor = active_monitors.iter().find(|monitor| {
            monitor
                .hardware_id
                .as_ref()
                .is_some_and(|hardware_id| hardware_id.eq_ignore_ascii_case(&display.hardware_id))
        });

        if !active_monitors.is_empty() && active_monitor.is_none() {
            continue;
        }

        let edid_info = parse_edid_info(&display.edid);
        let settings = active_monitor.and_then(|monitor| monitor.settings.as_ref());

        let hardware_id = display.hardware_id;

        displays.push(DisplayInfo {
            name: display
                .name
                .or_else(|| active_monitor.map(|monitor| monitor.name.clone()))
                .unwrap_or_else(|| hardware_id.clone()),
            vendor: edid_info.vendor,
            hardware_id: Some(hardware_id),
            device_id: active_monitor.map(|monitor| monitor.device_id.clone()),
            adapter_name: active_monitor.map(|monitor| monitor.adapter_name.clone()),
            adapter_device_name: active_monitor.map(|monitor| monitor.adapter_device_name.clone()),
            is_primary: active_monitor.map(|monitor| monitor.is_primary),
            current_resolution_width: settings.map(|settings| settings.width),
            current_resolution_height: settings.map(|settings| settings.height),
            refresh_rate_hz: settings.map(|settings| settings.refresh_rate_hz),
            bits_per_pixel: settings.map(|settings| settings.bits_per_pixel),
            position_x: settings.map(|settings| settings.position_x),
            position_y: settings.map(|settings| settings.position_y),
            manufacturer_id: edid_info.manufacturer_id,
            product_code: edid_info.product_code,
            serial_number: edid_info.serial_number,
            manufacture_week: edid_info.manufacture_week,
            manufacture_year: edid_info.manufacture_year,
            width_cm: edid_info.width_cm,
            height_cm: edid_info.height_cm,
            diagonal_inches: edid_info.diagonal_inches,
            edid: Some(bytes_to_hex(&display.edid)),
        });
    }

    for monitor in active_monitors {
        let already_collected = monitor.hardware_id.as_ref().is_some_and(|hardware_id| {
            displays.iter().any(|display| {
                display
                    .hardware_id
                    .as_ref()
                    .is_some_and(|display_hardware_id| {
                        display_hardware_id.eq_ignore_ascii_case(hardware_id)
                    })
            })
        });

        if !already_collected {
            displays.push(DisplayInfo::from_active_monitor(monitor.clone()));
        }
    }

    Ok(displays)
}

#[cfg(target_os = "windows")]
fn enumerate_edid_displays() -> HwResult<Vec<EdidDisplay>> {
    let display_key = RegKey::open_local_machine("SYSTEM\\CurrentControlSet\\Enum\\DISPLAY")?;
    let mut displays = Vec::new();

    for vendor_key_name in display_key.subkey_names()? {
        let vendor_key = match display_key.open_subkey(&vendor_key_name) {
            Ok(key) => key,
            Err(_) => continue,
        };

        for instance_key_name in vendor_key.subkey_names()? {
            let instance_key = match vendor_key.open_subkey(&instance_key_name) {
                Ok(key) => key,
                Err(_) => continue,
            };
            let parameters_key = match instance_key.open_subkey("Device Parameters") {
                Ok(key) => key,
                Err(_) => continue,
            };
            let edid = match parameters_key.binary_value("EDID") {
                Ok(value) if value.len() >= 128 => value,
                _ => continue,
            };

            displays.push(EdidDisplay {
                hardware_id: vendor_key_name.clone(),
                name: edid_display_name(&edid),
                edid,
            });
        }
    }

    Ok(displays)
}

#[cfg(target_os = "windows")]
struct RegKey(windows::Win32::System::Registry::HKEY);

#[cfg(target_os = "windows")]
impl RegKey {
    fn open_local_machine(path: &str) -> HwResult<Self> {
        use windows::core::PCWSTR;
        use windows::Win32::System::Registry::{RegOpenKeyExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ};

        let mut key = HKEY::default();
        let path_name = path.to_string();
        let path = wide_null_terminated(path);
        let status = unsafe {
            RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                PCWSTR(path.as_ptr()),
                0,
                KEY_READ,
                &mut key,
            )
        };

        if status.is_ok() {
            Ok(Self(key))
        } else {
            Err(format!(
                "cannot open registry key HKLM\\{path_name}: error {}",
                status.0
            ))
        }
    }

    fn open_subkey(&self, path: &str) -> HwResult<Self> {
        use windows::core::PCWSTR;
        use windows::Win32::System::Registry::{RegOpenKeyExW, HKEY, KEY_READ};

        let mut key = HKEY::default();
        let path = wide_null_terminated(path);
        let status = unsafe { RegOpenKeyExW(self.0, PCWSTR(path.as_ptr()), 0, KEY_READ, &mut key) };

        if status.is_ok() {
            Ok(Self(key))
        } else {
            Err(format!("cannot open registry subkey: error {}", status.0))
        }
    }

    fn subkey_names(&self) -> HwResult<Vec<String>> {
        use windows::core::PWSTR;
        use windows::Win32::Foundation::{ERROR_NO_MORE_ITEMS, ERROR_SUCCESS};
        use windows::Win32::System::Registry::RegEnumKeyExW;

        let mut names = Vec::new();
        let mut index = 0;

        loop {
            let mut buffer = [0u16; 256];
            let mut len = buffer.len() as u32;
            let status = unsafe {
                RegEnumKeyExW(
                    self.0,
                    index,
                    PWSTR(buffer.as_mut_ptr()),
                    &mut len,
                    None,
                    PWSTR::null(),
                    None,
                    None,
                )
            };

            if status == ERROR_NO_MORE_ITEMS {
                break;
            }

            if status != ERROR_SUCCESS {
                return Err(format!(
                    "cannot enumerate registry subkeys: error {}",
                    status.0
                ));
            }

            names.push(String::from_utf16_lossy(&buffer[..len as usize]));
            index += 1;
        }

        Ok(names)
    }

    fn binary_value(&self, name: &str) -> HwResult<Vec<u8>> {
        use windows::core::PCWSTR;
        use windows::Win32::Foundation::ERROR_SUCCESS;
        use windows::Win32::System::Registry::{RegQueryValueExW, REG_BINARY, REG_VALUE_TYPE};

        let name = wide_null_terminated(name);
        let mut value_type = REG_VALUE_TYPE::default();
        let mut len = 0;
        let status = unsafe {
            RegQueryValueExW(
                self.0,
                PCWSTR(name.as_ptr()),
                None,
                Some(&mut value_type),
                None,
                Some(&mut len),
            )
        };

        if status != ERROR_SUCCESS {
            return Err(format!(
                "cannot query registry value size: error {}",
                status.0
            ));
        }

        if value_type != REG_BINARY {
            return Err("registry value is not binary".to_string());
        }

        let mut value = vec![0; len as usize];
        let status = unsafe {
            RegQueryValueExW(
                self.0,
                PCWSTR(name.as_ptr()),
                None,
                Some(&mut value_type),
                Some(value.as_mut_ptr()),
                Some(&mut len),
            )
        };

        if status == ERROR_SUCCESS {
            value.truncate(len as usize);
            Ok(value)
        } else {
            Err(format!("cannot query registry value: error {}", status.0))
        }
    }
}

#[cfg(target_os = "windows")]
impl Drop for RegKey {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::System::Registry::RegCloseKey(self.0);
        }
    }
}

#[cfg(target_os = "windows")]
fn edid_display_name(edid: &[u8]) -> Option<String> {
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

#[cfg(target_os = "windows")]
fn parse_edid_info(edid: &[u8]) -> EdidInfo {
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

#[cfg(target_os = "windows")]
fn edid_manufacturer_id(edid: &[u8]) -> Option<String> {
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

#[cfg(target_os = "windows")]
fn edid_vendor_name(manufacturer_id: &str) -> Option<&'static str> {
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

#[cfg(target_os = "windows")]
fn non_zero_byte(value: u8) -> Option<u8> {
    if value == 0 {
        None
    } else {
        Some(value)
    }
}

#[cfg(target_os = "windows")]
fn non_zero_u16(value: u16) -> Option<u16> {
    if value == 0 {
        None
    } else {
        Some(value)
    }
}

#[cfg(target_os = "windows")]
fn non_zero_u32(value: u32) -> Option<u32> {
    if value == 0 {
        None
    } else {
        Some(value)
    }
}

#[cfg(target_os = "windows")]
fn diagonal_inches(width_cm: Option<u8>, height_cm: Option<u8>) -> Option<f32> {
    let width_cm = width_cm? as f32;
    let height_cm = height_cm? as f32;
    let diagonal = (width_cm.powi(2) + height_cm.powi(2)).sqrt() / 2.54;

    Some((diagonal * 10.0).round() / 10.0)
}

#[cfg(target_os = "windows")]
fn monitor_hardware_id(device_id: &str) -> Option<String> {
    if let Some((_, rest)) = device_id.split_once("DISPLAY#") {
        return rest.split('#').next().map(str::to_string);
    }

    let mut parts = device_id.split('\\');
    match (parts.next(), parts.next()) {
        (Some(prefix), Some(hardware_id)) if prefix.eq_ignore_ascii_case("MONITOR") => {
            Some(hardware_id.to_string())
        }
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut output = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }

    output
}

#[cfg(target_os = "windows")]
fn utf16_null_terminated_to_string(value: &[u16]) -> String {
    let len = value
        .iter()
        .position(|character| *character == 0)
        .unwrap_or(value.len());

    String::from_utf16_lossy(&value[..len]).trim().to_string()
}

#[cfg(target_os = "windows")]
fn wide_null_terminated(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn parses_edid_display_name() {
        let mut edid = vec![0; 128];
        edid[54..72].copy_from_slice(&[
            0x00, 0x00, 0x00, 0xfc, 0x00, b'O', b'd', b'y', b's', b's', b'e', b'y', b' ', b'G',
            b'5', b'2', b'A', 0x0a,
        ]);

        assert_eq!(edid_display_name(&edid), Some("Odyssey G52A".to_string()));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn parses_monitor_hardware_id() {
        assert_eq!(
            monitor_hardware_id(r"MONITOR\SAM71E7\{4d36e96e-e325-11ce-bfc1-08002be10318}"),
            Some("SAM71E7".to_string())
        );
        assert_eq!(
            monitor_hardware_id(
                r"\\?\DISPLAY#SAM71E7#5&22264ea&1&UID4353#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}"
            ),
            Some("SAM71E7".to_string())
        );
    }
}
