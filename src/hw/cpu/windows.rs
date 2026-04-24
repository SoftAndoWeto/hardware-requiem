use super::{compose_cpu_identifier, mhz_to_hz, normalize_cpu_brand, CpuInfo, HwResult};

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

fn read_cpu_string_value(value_name: &str) -> HwResult<String> {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::ERROR_SUCCESS;
    use windows::Win32::System::Registry::{
        HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_SZ, REG_VALUE_TYPE, RegOpenKeyExW,
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
        return Err(format!(
            "cannot query CPU registry value length: error {}",
            status.0
        ));
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
        return Err(format!(
            "cannot read CPU registry dword value: error {}",
            status.0
        ));
    }
    if value_type != REG_DWORD {
        return Err("CPU registry value has unexpected type".to_string());
    }

    Ok(value_data)
}

fn physical_core_count() -> usize {
    use windows::Win32::System::SystemInformation::{
        GetLogicalProcessorInformationEx, RelationProcessorCore,
        SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX,
    };

    let fallback = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);

    let mut required_len = 0u32;
    let _ = unsafe {
        GetLogicalProcessorInformationEx(RelationProcessorCore, None, &mut required_len)
    };
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

struct OwnedRegKey(windows::Win32::System::Registry::HKEY);

impl Drop for OwnedRegKey {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::System::Registry::RegCloseKey(self.0);
        }
    }
}

fn wide_null_terminated(value: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    std::ffi::OsStr::new(value)
        .encode_wide()
        .chain(Some(0))
        .collect()
}
