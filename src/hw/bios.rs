use std::thread;

use serde::{Deserialize, Serialize};

use super::HwResult;

/// Represents the BIOS information of the current system.
///
/// This struct is used to store and serialize the BIOS-related data retrieved from the system.
/// It contains the following fields:
/// - uuid: A string representing the unique identifier for the BIOS.
/// - manufacturer: A string representing the name of the BIOS manufacturer.
/// - name: A string containing the version and release date of the BIOS.
#[derive(Debug, Serialize, Deserialize)]
pub struct BiosInfo {
    pub uuid: String,
    pub manufacturer: String,
    pub name: String,
}

/// Represents the raw BIOS information retrieved from the WMI query.
///
/// This struct corresponds to the Win32_BIOS class in Windows Management Instrumentation (WMI).
/// It contains fields that represent various properties of the BIOS, including:
/// - SerialNumber: The serial number of the BIOS.
/// - Manufacturer: The manufacturer of the BIOS.
/// - Version: The version of the BIOS.
/// - ReleaseDate: The release date of the BIOS.
#[allow(non_snake_case, non_camel_case_types)]
#[derive(Deserialize, Debug)]
struct Win32_BIOS {
    pub SerialNumber: String,
    pub Manufacturer: String,
    pub Version: String,
    pub ReleaseDate: String,
}

/// Retrieves the BIOS information of the current system.
///
/// This function is designed to work on Windows systems and uses the `wmi` crate to interact with
/// the Windows Management Instrumentation (WMI). It spawns a new thread to perform the WMI query
/// and then collects the results.
///
/// # Returns
///
/// Returns a `BiosInfo` struct containing the following fields:
/// - `uuid`: A string representing the BIOS UUID.
/// - `manufacturer`: A string representing the BIOS manufacturer.
/// - `name`: A string representing the BIOS version and release date.
#[cfg(target_os = "windows")]
pub fn get_bios_info() -> HwResult<BiosInfo> {
    use wmi::*;
    thread::spawn(|| {
        let wmi_con = WMIConnection::new(COMLibrary::new().map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;

        let results: Vec<Win32_BIOS> = wmi_con.query().map_err(|error| error.to_string())?;
        let win32_bios = results
            .first()
            .ok_or_else(|| "WMI Win32_BIOS returned no records".to_string())?;

        Ok(BiosInfo {
            uuid: win32_bios.SerialNumber.clone(),
            manufacturer: win32_bios.Manufacturer.clone(),
            name: format!("{} {}", win32_bios.Version, win32_bios.ReleaseDate),
        })
    })
    .join()
    .map_err(|_| "BIOS collector thread panicked".to_string())?
}
