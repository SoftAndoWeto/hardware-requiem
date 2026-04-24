use serde::{Deserialize, Serialize};

use super::HwResult;

/// Represents detailed information about the CPU.
///
/// This struct is used to store and serialize CPU-related data retrieved from the system.
/// It contains the following fields:
/// - name: A string representing the brand name of the CPU.
/// - identifier: A string that uniquely identifies the CPU, typically combining vendor ID and CPU
///   name.
/// - processor_id: An optional string that can hold the processor ID, if available.
/// - vendor_frequency: A 64-bit unsigned integer representing the CPU's vendor frequency in Hz.
/// - physical_processor_count: A usize indicating the number of physical processors present in the
///   system.
#[derive(Debug, Serialize, Deserialize)]
pub struct CpuInfo {
    pub name: String,
    pub identifier: String,
    #[serde(rename = "processorId")]
    pub processor_id: Option<String>,
    #[serde(rename = "vendorFrequency")]
    pub vendor_frequency: u64,
    #[serde(rename = "physicalProcessorCount")]
    pub physical_processor_count: usize,
}

/// Retrieves detailed information about the CPU.
///
/// On Windows, this collector reads CPU metadata from the registry and uses
/// `GetLogicalProcessorInformationEx` to estimate physical core count.
#[cfg(target_os = "windows")]
pub fn get_cpu_info() -> HwResult<CpuInfo> {
    let name = normalize_cpu_brand(&read_cpu_string_value("ProcessorNameString")?);
    let vendor_id = read_cpu_string_value("VendorIdentifier")?;
    let raw_identifier = read_cpu_string_value("Identifier")?;
    let frequency_mhz = read_cpu_dword_value("~MHz")? as u64;

    Ok(CpuInfo {
        name,
        identifier: compose_cpu_identifier(&vendor_id, &raw_identifier),
        processor_id: None,
        vendor_frequency: mhz_to_hz(frequency_mhz),
        physical_processor_count: physical_core_count(),
    })
}

#[cfg(target_os = "linux")]
pub fn get_cpu_info() -> HwResult<CpuInfo> {
    let content = std::fs::read_to_string("/proc/cpuinfo")
        .map_err(|e| format!("cannot read /proc/cpuinfo: {e}"))?;
    parse_cpu_info_from_procfs(&content)
}

#[cfg(not(any(windows, target_os = "linux")))]
pub fn get_cpu_info() -> HwResult<CpuInfo> {
    Err("cpu collection is not implemented for this platform".to_string())
}

#[cfg(target_os = "linux")]
fn parse_cpu_info_from_procfs(content: &str) -> HwResult<CpuInfo> {
    parse_cpu_info_from_procfs_with_freq(content, read_cpu_max_freq_hz().ok())
}

#[cfg(target_os = "linux")]
fn parse_cpu_info_from_procfs_with_freq(
    content: &str,
    max_freq_hz: Option<u64>,
) -> HwResult<CpuInfo> {
    use std::collections::HashSet;

    let mut name: Option<String> = None;
    let mut vendor_id = String::new();
    let mut cpu_family = String::new();
    let mut model_num = String::new();
    let mut stepping = String::new();
    let mut cpu_mhz: Option<f64> = None;
    let mut cores: HashSet<(String, String)> = HashSet::new();

    for block in content.split("\n\n") {
        let mut physical_id = String::new();
        let mut core_id = String::new();

        for line in block.lines() {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim();

            match key {
                "model name" if name.is_none() => name = Some(normalize_cpu_brand(value)),
                "vendor_id" if vendor_id.is_empty() => vendor_id = value.to_string(),
                "cpu family" if cpu_family.is_empty() => cpu_family = value.to_string(),
                "model" if model_num.is_empty() => model_num = value.to_string(),
                "stepping" if stepping.is_empty() => stepping = value.to_string(),
                "cpu MHz" if cpu_mhz.is_none() => cpu_mhz = value.parse().ok(),
                "physical id" => physical_id = value.to_string(),
                "core id" => core_id = value.to_string(),
                _ => {}
            }
        }

        if !physical_id.is_empty() || !core_id.is_empty() {
            cores.insert((physical_id, core_id));
        }
    }

    let name = name.ok_or("model name not found in /proc/cpuinfo")?;
    let raw_identifier = format!("Family {cpu_family} Model {model_num} Stepping {stepping}");
    let identifier = compose_cpu_identifier(&vendor_id, &raw_identifier);

    let vendor_frequency = max_freq_hz
        .or_else(|| cpu_mhz.map(|mhz| mhz_to_hz(mhz as u64)))
        .ok_or("cannot determine CPU frequency")?;

    let physical_processor_count = if cores.is_empty() {
        content
            .lines()
            .filter(|l| l.starts_with("processor"))
            .count()
            .max(1)
    } else {
        cores.len()
    };

    Ok(CpuInfo {
        name,
        identifier,
        processor_id: None,
        vendor_frequency,
        physical_processor_count,
    })
}

#[cfg(target_os = "linux")]
fn read_cpu_max_freq_hz() -> HwResult<u64> {
    let content =
        std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq")
            .map_err(|e| format!("cannot read cpuinfo_max_freq: {e}"))?;
    let khz: u64 = content
        .trim()
        .parse()
        .map_err(|e| format!("cannot parse cpuinfo_max_freq: {e}"))?;
    Ok(khz.saturating_mul(1_000))
}

fn normalize_cpu_brand(brand: &str) -> String {
    brand.trim().to_string()
}

fn compose_cpu_identifier(vendor_id: &str, cpu_name: &str) -> String {
    format!("{vendor_id} - {cpu_name}")
}

fn mhz_to_hz(mhz: u64) -> u64 {
    mhz.saturating_mul(1_000_000)
}

#[cfg(target_os = "windows")]
fn read_cpu_string_value(value_name: &str) -> HwResult<String> {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::ERROR_SUCCESS;
    use windows::Win32::System::Registry::{
        HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_SZ, REG_VALUE_TYPE, RegOpenKeyExW, RegQueryValueExW,
    };

    const CPU_REG_PATH: &str = "HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0";

    let mut key = HKEY::default();
    let key_path = wide_null_terminated(CPU_REG_PATH);
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR(key_path.as_ptr()),
            Some(0),
            KEY_READ,
            &mut key,
        )
    };
    if status != ERROR_SUCCESS {
        return Err(format!(
            "cannot open CPU registry key HKLM\\{CPU_REG_PATH}: error {}",
            status.0
        ));
    }

    let reg_key = OwnedRegKey(key);
    let value_name = wide_null_terminated(value_name);
    let mut value_type = REG_VALUE_TYPE(0);
    let mut value_len = 0u32;
    let status = unsafe {
        RegQueryValueExW(
            reg_key.0,
            PCWSTR(value_name.as_ptr()),
            None,
            Some(&mut value_type),
            None,
            Some(&mut value_len),
        )
    };
    if status != ERROR_SUCCESS {
        return Err(format!("cannot query CPU registry value length: error {}", status.0));
    }
    if value_type != REG_SZ {
        return Err("CPU registry value has unexpected type".to_string());
    }
    if value_len == 0 {
        return Err("CPU registry value is empty".to_string());
    }

    let mut buffer = vec![0u8; value_len as usize];
    let status = unsafe {
        RegQueryValueExW(
            reg_key.0,
            PCWSTR(value_name.as_ptr()),
            None,
            Some(&mut value_type),
            Some(buffer.as_mut_ptr()),
            Some(&mut value_len),
        )
    };
    if status != ERROR_SUCCESS {
        return Err(format!("cannot read CPU registry value: error {}", status.0));
    }

    let mut utf16 = Vec::with_capacity(buffer.len() / 2);
    for chunk in buffer.chunks_exact(2) {
        utf16.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    while matches!(utf16.last(), Some(0)) {
        utf16.pop();
    }

    Ok(String::from_utf16_lossy(&utf16).trim().to_string())
}

#[cfg(target_os = "windows")]
fn read_cpu_dword_value(value_name: &str) -> HwResult<u32> {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::ERROR_SUCCESS;
    use windows::Win32::System::Registry::{
        HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_DWORD, REG_VALUE_TYPE, RegOpenKeyExW,
        RegQueryValueExW,
    };

    const CPU_REG_PATH: &str = "HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0";

    let mut key = HKEY::default();
    let key_path = wide_null_terminated(CPU_REG_PATH);
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR(key_path.as_ptr()),
            Some(0),
            KEY_READ,
            &mut key,
        )
    };
    if status != ERROR_SUCCESS {
        return Err(format!(
            "cannot open CPU registry key HKLM\\{CPU_REG_PATH}: error {}",
            status.0
        ));
    }

    let reg_key = OwnedRegKey(key);
    let value_name = wide_null_terminated(value_name);
    let mut value_type = REG_VALUE_TYPE(0);
    let mut value_len = std::mem::size_of::<u32>() as u32;
    let mut value_data = 0u32;

    let status = unsafe {
        RegQueryValueExW(
            reg_key.0,
            PCWSTR(value_name.as_ptr()),
            None,
            Some(&mut value_type),
            Some((&mut value_data as *mut u32).cast::<u8>()),
            Some(&mut value_len),
        )
    };
    if status != ERROR_SUCCESS {
        return Err(format!("cannot read CPU registry dword value: error {}", status.0));
    }
    if value_type != REG_DWORD {
        return Err("CPU registry value has unexpected type".to_string());
    }

    Ok(value_data)
}

#[cfg(target_os = "windows")]
fn physical_core_count() -> usize {
    use windows::Win32::System::SystemInformation::{
        GetLogicalProcessorInformationEx, RelationProcessorCore,
        SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX,
    };

    let fallback = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);

    let mut required_len = 0u32;
    let _ = unsafe { GetLogicalProcessorInformationEx(RelationProcessorCore, None, &mut required_len) };
    if required_len == 0 {
        return fallback;
    }

    let mut buffer = vec![0u8; required_len as usize];
    if unsafe {
        GetLogicalProcessorInformationEx(
            RelationProcessorCore,
            Some(buffer.as_mut_ptr().cast::<SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX>()),
            &mut required_len,
        )
    }
    .is_err()
    {
        return fallback;
    }

    let mut count = 0usize;
    let mut offset = 0usize;
    while offset + std::mem::size_of::<SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX>() <= buffer.len() {
        let info = unsafe {
            &*(buffer
                .as_ptr()
                .add(offset)
                .cast::<SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX>())
        };

        if info.Relationship == RelationProcessorCore {
            count += 1;
        }

        if info.Size == 0 {
            break;
        }
        offset = offset.saturating_add(info.Size as usize);
    }

    if count == 0 { fallback } else { count }
}

#[cfg(target_os = "windows")]
struct OwnedRegKey(windows::Win32::System::Registry::HKEY);

#[cfg(target_os = "windows")]
impl Drop for OwnedRegKey {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::System::Registry::RegCloseKey(self.0);
        }
    }
}

#[cfg(target_os = "windows")]
fn wide_null_terminated(value: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    std::ffi::OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

#[cfg(test)]
mod tests;
