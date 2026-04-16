use std::fmt;

pub type Result<T> = std::result::Result<T, HardwareInfoError>;

#[derive(Debug)]
pub enum HardwareInfoError {
    PlatformUnsupported(&'static str),
    WindowsApi {
        function: &'static str,
        message: String,
    },
}

impl fmt::Display for HardwareInfoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlatformUnsupported(platform) => {
                write!(f, "hardware information is not implemented for {platform}")
            }
            Self::WindowsApi { function, message } => {
                write!(f, "{function} failed: {message}")
            }
        }
    }
}

impl std::error::Error for HardwareInfoError {}
