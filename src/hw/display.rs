use display_info::DisplayInfo as DI;
use serde::{Deserialize, Serialize};

use super::HwResult;

/// Represents information about a display device.
///
/// This struct is used to store and serialize data related to display devices connected to the
/// system.
/// It contains the following fields:
/// - name: A string representing the name of the display device.
/// - edid: An optional string that holds the Extended Display Identification Data (EDID) for the
///   display. This field can be None if the EDID is not available or not applicable.
#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub name: String,
    pub edid: Option<String>, // none
}

/// Retrieves a list of display information.
///
/// This function uses the `display_info` crate to fetch detailed information about each display
/// connected to the system.
///
/// # Returns
///
/// A vector of `DisplayInfo` structs, where each struct contains the name of the display and
/// `None` for the EDID.
/// If an error occurs while loading display information, the function will panic with the message
/// "Cannot load display infos".
pub fn get_display() -> HwResult<Vec<DisplayInfo>> {
    let display_infos = DI::all().map_err(|error| error.to_string())?;
    let mut display_info_list = vec![];

    for display in display_infos {
        display_info_list.push(DisplayInfo {
            name: display.name,
            edid: None,
        });
    }

    Ok(display_info_list)
}
