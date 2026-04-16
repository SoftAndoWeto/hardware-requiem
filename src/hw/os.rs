use serde::{Deserialize, Serialize};
use sysinfo::System;

use super::HwResult;

/// Represents information about the operating system.
///
/// This struct is used to store and serialize data related to the operating system.
/// It contains the following fields:
/// - manufacturer: An optional string representing the manufacturer of the operating system.
///   Currently, this field is always None.
/// - family: A string representing the family of the operating system.
/// - version: A string representing the version of the operating system.
/// - bussines: An optional boolean indicating whether the operating system is a business
///   edition. Currently, this field is always None.
#[derive(Debug, Serialize, Deserialize)]
pub struct OsInfo {
    pub manufacturer: Option<String>,
    pub family: String,
    pub version: String,
    pub bussines: Option<bool>,
}

/// Retrieves operating system information.
///
/// This function uses the `sysinfo` crate to gather information about the operating system.
/// It returns an instance of the `OsInfo` struct containing the following fields:
/// - `manufacturer`: An optional string representing the manufacturer of the operating system.
///   Currently, this field is always `None`.
/// - `family`: A string representing the family of the operating system.
/// - `version`: A string representing the version of the operating system.
/// - `bussines`: An optional boolean indicating whether the operating system is a business
///   edition. Currently, this field is always `None`.
///
/// # Errors
///
/// This function may return errors if it is unable to retrieve the operating system name or
/// version. In such cases, it will panic with an error message.
pub fn get_os_info() -> HwResult<OsInfo> {
    Ok(OsInfo {
        manufacturer: None,
        family: System::name().unwrap_or_else(|| std::env::consts::OS.to_string()),
        version: System::long_os_version().unwrap_or_else(|| "unknown".to_string()),
        bussines: None,
    })
}
