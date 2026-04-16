use serde::{Deserialize, Serialize};
use std::thread;
use wmi::*;

use super::HwResult;

/// Represents information about the motherboard.
///
/// This struct is used to store and serialize data related to the motherboard
/// as retrieved from the Windows Management Instrumentation (WMI). It contains
/// the following fields:
/// - manufacturer: A string representing the manufacturer of the motherboard.
/// - serial_number: A string representing the serial number of the motherboard.
#[derive(Debug, Serialize, Deserialize)]
pub struct MotherboardInfo {
    pub manufacturer: String,
    #[serde(rename = "serialNumber")]
    pub serial_number: String,
}

/// Represents the WMI representation of the motherboard base board.
///
/// This struct is used to deserialize data from WMI queries related to the
/// motherboard's base board. It contains the following fields:
/// - SerialNumber: A string representing the serial number of the base board.
/// - Manufacturer: A string representing the manufacturer of the base board.
#[allow(non_snake_case, non_camel_case_types)]
#[derive(Deserialize, Debug)]
struct Win32_BaseBoard {
    pub SerialNumber: String,
    pub Manufacturer: String,
}

/// Retrieves the motherboard information using the Windows Management Instrumentation (WMI).
///
/// This function spawns a new thread to perform the WMI query and retrieves the motherboard's
/// serial number and manufacturer. The retrieved information is then returned as a
/// `MotherboardInfo` struct.
///
/// # Returns
///
/// * `MotherboardInfo` - A struct containing the motherboard's serial number and manufacturer.
#[cfg(target_os = "windows")]
pub fn get_motherboard_info() -> HwResult<MotherboardInfo> {
    thread::spawn(|| {
        let wmi_con = WMIConnection::new(COMLibrary::new().map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;

        let results: Vec<Win32_BaseBoard> = wmi_con.query().map_err(|error| error.to_string())?;
        let win32_motherboard = results
            .first()
            .ok_or_else(|| "WMI Win32_BaseBoard returned no records".to_string())?;

        Ok(MotherboardInfo {
            serial_number: win32_motherboard.SerialNumber.clone(),
            manufacturer: win32_motherboard.Manufacturer.clone(),
        })
    })
    .join()
    .map_err(|_| "motherboard collector thread panicked".to_string())?
}
