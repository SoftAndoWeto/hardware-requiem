use crate::{HardwareInfo, HardwareInfoError, Result};

pub(crate) fn collect() -> Result<HardwareInfo> {
    Err(HardwareInfoError::PlatformUnsupported(std::env::consts::OS))
}

pub(crate) fn collect_os_info() -> Result<crate::OsInfo> {
    Err(HardwareInfoError::PlatformUnsupported(std::env::consts::OS))
}
