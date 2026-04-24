use serde::{Deserialize, Serialize};

use super::HwResult;

mod edid;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

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

#[cfg(windows)]
pub use self::windows::get_display;
#[cfg(target_os = "linux")]
pub use self::linux::get_display;

#[cfg(not(any(windows, target_os = "linux")))]
pub fn get_display() -> HwResult<Vec<DisplayInfo>> {
    Ok(Vec::new())
}

#[cfg(test)]
mod tests;
