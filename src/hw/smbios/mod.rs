use super::HwResult;

mod parser;

#[cfg(windows)]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(windows)]
pub use self::windows::{read_raw_smbios_table, smbios_table_bytes};

#[cfg(target_os = "linux")]
pub use self::linux::{read_raw_smbios_table, smbios_table_bytes};

pub use parser::{join_non_empty, parse_smbios_structures, SmbiosStructure};

#[cfg(test)]
mod tests;
