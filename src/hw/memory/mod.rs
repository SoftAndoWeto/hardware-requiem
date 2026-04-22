use super::HwResult;

mod parser;

#[cfg(windows)]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(windows)]
pub use self::windows::get_memory_info;

#[cfg(target_os = "linux")]
pub use self::linux::get_memory_info;

#[cfg(not(any(windows, target_os = "linux")))]
pub fn get_memory_info() -> HwResult<Vec<parser::MemoryInfo>> {
    Err("memory collection is not implemented for this platform".to_string())
}

pub use parser::MemoryInfo;

#[cfg(test)]
mod tests;
