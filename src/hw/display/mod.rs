//! Display monitor detection and EDID metadata.
//!
//! Windows: monitors are enumerated via `EnumDisplayDevicesW` and EDID is read
//! from `HKLM\SYSTEM\CurrentControlSet\Enum\DISPLAY`.
//!
//! Linux: connected DRM connectors are discovered under `/sys/class/drm/card*-*/`
//! and their `edid` sysfs files are parsed directly.

use serde::{Deserialize, Serialize};

use super::HwResult;

mod edid;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

/// Information about a connected display monitor.
///
/// Most EDID fields (`manufacturer_id`, `product_code`, dimensions, etc.) are
/// available on both platforms. Fields like `is_primary`, `refresh_rate_hz`,
/// and `adapter_name` are Windows-only and will be `None` on Linux.
#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayInfo {
    /// Display name from the EDID descriptor, OS device string, or connector
    /// name — whichever is available first.
    pub name: String,
    /// Vendor name resolved from the EDID manufacturer ID (e.g. `"Samsung"`).
    pub vendor: Option<String>,
    /// Windows hardware ID, e.g. `"SAM71E7"`. `None` on Linux.
    #[serde(rename = "hardwareId")]
    pub hardware_id: Option<String>,
    /// Full device instance path reported by Windows.
    #[serde(rename = "deviceId")]
    pub device_id: Option<String>,
    /// GPU adapter friendly name. Windows only.
    #[serde(rename = "adapterName")]
    pub adapter_name: Option<String>,
    /// GPU adapter device path, e.g. `"\\\\.\\DISPLAY1"`. Windows only.
    #[serde(rename = "adapterDeviceName")]
    pub adapter_device_name: Option<String>,
    /// Whether this is the primary display. Windows only.
    #[serde(rename = "isPrimary")]
    pub is_primary: Option<bool>,
    /// Current horizontal resolution in pixels. Windows only.
    #[serde(rename = "currentResolutionWidth")]
    pub current_resolution_width: Option<u32>,
    /// Current vertical resolution in pixels. Windows only.
    #[serde(rename = "currentResolutionHeight")]
    pub current_resolution_height: Option<u32>,
    /// Refresh rate in Hz. Windows only.
    #[serde(rename = "refreshRateHz")]
    pub refresh_rate_hz: Option<u32>,
    /// Color depth in bits per pixel. Windows only.
    #[serde(rename = "bitsPerPixel")]
    pub bits_per_pixel: Option<u32>,
    /// X position of the monitor in the virtual desktop. Windows only.
    #[serde(rename = "positionX")]
    pub position_x: Option<i32>,
    /// Y position of the monitor in the virtual desktop. Windows only.
    #[serde(rename = "positionY")]
    pub position_y: Option<i32>,
    /// Three-letter EDID manufacturer ID, e.g. `"SAM"`.
    #[serde(rename = "manufacturerId")]
    pub manufacturer_id: Option<String>,
    /// Manufacturer product code from EDID bytes 8–9.
    #[serde(rename = "productCode")]
    pub product_code: Option<u16>,
    /// 32-bit serial number from EDID bytes 12–15. `None` if the field is zero.
    #[serde(rename = "serialNumber")]
    pub serial_number: Option<u32>,
    /// Week of manufacture (1–53), if specified in EDID.
    #[serde(rename = "manufactureWeek")]
    pub manufacture_week: Option<u8>,
    /// Year of manufacture, e.g. `2023`. `None` if not specified in EDID.
    #[serde(rename = "manufactureYear")]
    pub manufacture_year: Option<u16>,
    /// Physical width in centimetres from EDID.
    #[serde(rename = "widthCm")]
    pub width_cm: Option<u8>,
    /// Physical height in centimetres from EDID.
    #[serde(rename = "heightCm")]
    pub height_cm: Option<u8>,
    /// Diagonal size in inches, computed from `width_cm` and `height_cm`.
    #[serde(rename = "diagonalInches")]
    pub diagonal_inches: Option<f32>,
    /// Raw EDID blob as an uppercase hex string.
    pub edid: Option<String>,
}

/// Returns information about connected display monitors.
///
/// On Windows combines GDI enumeration with EDID data from the registry.
/// On Linux reads EDID from `/sys/class/drm/card*-*/edid` for connectors
/// with `status == "connected"`.
#[cfg(windows)]
pub use self::windows::get_display;
#[cfg(target_os = "linux")]
pub use self::linux::get_display;

/// Returns information about connected display monitors.
///
/// Always returns an empty list on unsupported platforms.
#[cfg(not(any(windows, target_os = "linux")))]
pub fn get_display() -> HwResult<Vec<DisplayInfo>> {
    Ok(Vec::new())
}

#[cfg(test)]
mod tests;
