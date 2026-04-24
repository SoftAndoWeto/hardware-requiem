use super::edid::{bytes_to_hex, edid_display_name, parse_edid_info};
use super::{DisplayInfo, HwResult};

pub fn get_display() -> HwResult<Vec<DisplayInfo>> {
    let mut displays: Vec<DisplayInfo> = std::fs::read_dir("/sys/class/drm")
        .map_err(|e| format!("cannot read /sys/class/drm: {e}"))?
        .flatten()
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            is_drm_connector(&name)
        })
        .filter_map(|entry| parse_drm_connector(&entry.path()))
        .collect();

    displays.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(displays)
}

pub(super) fn is_drm_connector(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("card") else {
        return false;
    };
    let dash_pos = rest.find('-').unwrap_or(rest.len());
    rest[..dash_pos].chars().all(|c| c.is_ascii_digit()) && dash_pos < rest.len()
}

fn parse_drm_connector(path: &std::path::Path) -> Option<DisplayInfo> {
    let status = std::fs::read_to_string(path.join("status")).ok()?;
    if status.trim() != "connected" {
        return None;
    }

    let edid = std::fs::read(path.join("edid")).ok()?;
    if edid.len() < 128 {
        return None;
    }

    let dir_name = path.file_name()?.to_string_lossy().to_string();
    let connector_name = dir_name
        .find('-')
        .map(|pos| dir_name[pos + 1..].to_string())
        .unwrap_or_else(|| dir_name.clone());

    let name = edid_display_name(&edid).unwrap_or_else(|| connector_name.clone());
    let edid_info = parse_edid_info(&edid);

    Some(DisplayInfo {
        name,
        vendor: edid_info.vendor,
        hardware_id: None,
        device_id: None,
        adapter_name: None,
        adapter_device_name: None,
        is_primary: None,
        current_resolution_width: None,
        current_resolution_height: None,
        refresh_rate_hz: None,
        bits_per_pixel: None,
        position_x: None,
        position_y: None,
        manufacturer_id: edid_info.manufacturer_id,
        product_code: edid_info.product_code,
        serial_number: edid_info.serial_number,
        manufacture_week: edid_info.manufacture_week,
        manufacture_year: edid_info.manufacture_year,
        width_cm: edid_info.width_cm,
        height_cm: edid_info.height_cm,
        diagonal_inches: edid_info.diagonal_inches,
        edid: Some(bytes_to_hex(&edid)),
    })
}
