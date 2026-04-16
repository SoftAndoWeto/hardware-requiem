use crate::{HardwareInfo, HardwareInfoError, Result};

pub(crate) fn collect() -> Result<HardwareInfo> {
    Err(HardwareInfoError::PlatformUnsupported(std::env::consts::OS))
}
