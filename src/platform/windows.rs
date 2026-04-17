use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use windows::core::PCWSTR;
use windows::Wdk::System::SystemServices::RtlGetVersion;
use windows::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives};
use windows::Win32::System::SystemInformation::{
    GetNativeSystemInfo, GlobalMemoryStatusEx, MEMORYSTATUSEX, OSVERSIONINFOW,
    PROCESSOR_ARCHITECTURE, PROCESSOR_ARCHITECTURE_AMD64, PROCESSOR_ARCHITECTURE_ARM,
    PROCESSOR_ARCHITECTURE_ARM64, PROCESSOR_ARCHITECTURE_INTEL, SYSTEM_INFO,
};

use crate::{
    CpuArchitecture, CpuInfo, DiskInfo, DiskKind, HardwareInfo, HardwareInfoError, MemoryInfo,
    OsInfo, Result,
};

pub(crate) fn collect() -> Result<HardwareInfo> {
    Ok(HardwareInfo {
        os: collect_os(),
        cpu: collect_cpu(),
        memory: collect_memory()?,
        disks: collect_disks()?,
    })
}

pub(crate) fn collect_os_info() -> Result<OsInfo> {
    Ok(collect_os())
}

fn collect_os() -> OsInfo {
    OsInfo {
        family: std::env::consts::FAMILY.to_string(),
        name: Some("Windows".to_string()),
        version: windows_major_version(),
    }
}

fn windows_major_version() -> Option<String> {
    let mut version_info = OSVERSIONINFOW {
        dwOSVersionInfoSize: std::mem::size_of::<OSVERSIONINFOW>() as u32,
        ..Default::default()
    };

    let status = unsafe { RtlGetVersion(&mut version_info) };
    if !status.is_ok() {
        return None;
    }

    let major = version_info.dwMajorVersion;
    let build = version_info.dwBuildNumber;

    if major == 10 && build >= 22_000 {
        Some("11".to_string())
    } else if major == 10 {
        Some("10".to_string())
    } else {
        Some(major.to_string())
    }
}

fn collect_cpu() -> CpuInfo {
    let mut system_info = SYSTEM_INFO::default();

    unsafe {
        GetNativeSystemInfo(&mut system_info);
    }

    let processor_architecture = unsafe { system_info.Anonymous.Anonymous.wProcessorArchitecture };

    CpuInfo {
        architecture: cpu_architecture(processor_architecture),
        logical_cores: std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(system_info.dwNumberOfProcessors.max(1) as usize),
        vendor_or_brand: std::env::var("PROCESSOR_IDENTIFIER").ok(),
    }
}

fn cpu_architecture(raw: PROCESSOR_ARCHITECTURE) -> CpuArchitecture {
    if raw == PROCESSOR_ARCHITECTURE_INTEL {
        CpuArchitecture::X86
    } else if raw == PROCESSOR_ARCHITECTURE_AMD64 {
        CpuArchitecture::X86_64
    } else if raw == PROCESSOR_ARCHITECTURE_ARM {
        CpuArchitecture::Arm
    } else if raw == PROCESSOR_ARCHITECTURE_ARM64 {
        CpuArchitecture::Aarch64
    } else {
        CpuArchitecture::Unknown
    }
}

fn collect_memory() -> Result<MemoryInfo> {
    let mut status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };

    unsafe {
        GlobalMemoryStatusEx(&mut status).map_err(|error| HardwareInfoError::WindowsApi {
            function: "GlobalMemoryStatusEx",
            message: error.message(),
        })?;
    }

    Ok(MemoryInfo {
        total_physical_bytes: status.ullTotalPhys,
        available_physical_bytes: status.ullAvailPhys,
    })
}

fn collect_disks() -> Result<Vec<DiskInfo>> {
    let mask = unsafe { GetLogicalDrives() };

    if mask == 0 {
        return Err(HardwareInfoError::WindowsApi {
            function: "GetLogicalDrives",
            message: windows::core::Error::from_thread().message(),
        });
    }

    let mut disks = Vec::new();

    for index in 0..26 {
        if mask & (1 << index) == 0 {
            continue;
        }

        let mount_point = format!("{}:\\", (b'A' + index as u8) as char);
        let wide_mount_point = wide_null(&mount_point);
        let path = PCWSTR(wide_mount_point.as_ptr());
        let kind = disk_kind(unsafe { GetDriveTypeW(path) });

        if matches!(kind, DiskKind::Unknown) {
            continue;
        }

        let mut free_bytes_available = 0u64;
        let mut total_bytes = 0u64;
        let mut total_free_bytes = 0u64;

        let result = unsafe {
            GetDiskFreeSpaceExW(
                path,
                Some(&mut free_bytes_available),
                Some(&mut total_bytes),
                Some(&mut total_free_bytes),
            )
        };

        if result.is_err() {
            continue;
        }

        disks.push(DiskInfo {
            mount_point,
            kind,
            total_bytes,
            free_bytes: total_free_bytes,
        });
    }

    Ok(disks)
}

fn disk_kind(raw: u32) -> DiskKind {
    match raw {
        3 => DiskKind::Fixed,
        2 => DiskKind::Removable,
        4 => DiskKind::Network,
        5 => DiskKind::Optical,
        6 => DiskKind::RamDisk,
        0 | 1 => DiskKind::Unknown,
        _ => DiskKind::Unknown,
    }
}

fn wide_null(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}
