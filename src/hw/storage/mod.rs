use std::{ffi::CString, ffi::OsStr, mem::size_of, os::windows::ffi::OsStrExt};

use serde::{Deserialize, Serialize};
use windows::{
    core::{PCSTR, PCWSTR},
    Win32::Foundation::{CloseHandle, HANDLE},
    Win32::Storage::FileSystem::{
        CreateFileW, GetDiskFreeSpaceExA, GetLogicalDriveStringsW, GetVolumeInformationA,
        FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    },
    Win32::System::{
        Ioctl::{
            PropertyStandardQuery, StorageDeviceProperty, DISK_GEOMETRY_EX, GET_LENGTH_INFORMATION,
            IOCTL_DISK_GET_DRIVE_GEOMETRY_EX, IOCTL_DISK_GET_LENGTH_INFO,
            IOCTL_STORAGE_QUERY_PROPERTY, STORAGE_DEVICE_DESCRIPTOR, STORAGE_PROPERTY_QUERY,
        },
        IO::DeviceIoControl,
    },
};

use super::HwResult;

/// Represents information about a storage device.
///
/// This struct is used to store and serialize data related to connected storage devices,
/// such as hard drives, SSDs, and USB drives. It contains the following fields:
///
/// - name: A string representing the name of the storage device.
/// - model: A string representing the model of the storage device.
/// - serial_number: A string representing the serial number of the storage device.
/// - size: A 64-bit unsigned integer representing the total size of the storage device in bytes.
#[derive(Debug, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub model: String,
    #[serde(rename = "serialNumber")]
    pub serial_number: String,
    pub size: u64,
}

/// Retrieves information about physical storage devices.
pub fn get_storage() -> HwResult<Vec<DiskInfo>> {
    get_physical_storage()
}

fn get_physical_storage() -> HwResult<Vec<DiskInfo>> {
    let mut disks = Vec::new();
    let mut errors = Vec::new();

    for index in 0..32 {
        match query_physical_drive(index) {
            Ok(Some(disk)) => disks.push(disk),
            Ok(None) => {}
            Err(error) => errors.push(format!("PhysicalDrive{index}: {error}")),
        }
    }

    if disks.is_empty() {
        Err(if errors.is_empty() {
            "no physical drives were found".to_string()
        } else {
            errors.join("; ")
        })
    } else {
        Ok(disks)
    }
}

fn query_physical_drive(index: u32) -> HwResult<Option<DiskInfo>> {
    let path = format!("\\\\.\\PhysicalDrive{index}");
    let handle = match open_physical_drive(&path, FILE_GENERIC_READ.0) {
        Ok(Some(handle)) => handle,
        Ok(None) => return Ok(None),
        Err(_) => match open_physical_drive(&path, 0)? {
            Some(handle) => handle,
            None => return Ok(None),
        },
    };

    let descriptor = query_storage_descriptor(handle.0)?;
    let size = query_drive_size(handle.0)?;

    Ok(Some(DiskInfo {
        name: path,
        model: read_descriptor_string(&descriptor, |descriptor| descriptor.ProductIdOffset),
        serial_number: read_descriptor_string(&descriptor, |descriptor| {
            descriptor.SerialNumberOffset
        }),
        size,
    }))
}

fn open_physical_drive(path: &str, desired_access: u32) -> HwResult<Option<OwnedHandle>> {
    let wide_path = wide_null(path);

    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            desired_access,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            Some(HANDLE(std::ptr::null_mut())),
        )
    };

    match handle {
        Ok(handle) => Ok(Some(OwnedHandle(handle))),
        Err(error) => {
            let message = error.message();
            if is_not_found_message(&message) {
                Ok(None)
            } else {
                Err(message)
            }
        }
    }
}

fn query_storage_descriptor(handle: HANDLE) -> HwResult<Vec<u8>> {
    let query = STORAGE_PROPERTY_QUERY {
        PropertyId: StorageDeviceProperty,
        QueryType: PropertyStandardQuery,
        AdditionalParameters: [0],
    };

    let mut output = vec![0u8; 4096];
    let mut bytes_returned = 0u32;

    unsafe {
        DeviceIoControl(
            handle,
            IOCTL_STORAGE_QUERY_PROPERTY,
            Some((&query as *const STORAGE_PROPERTY_QUERY).cast()),
            size_of::<STORAGE_PROPERTY_QUERY>() as u32,
            Some(output.as_mut_ptr().cast()),
            output.len() as u32,
            Some(&mut bytes_returned),
            None,
        )
        .map_err(|error| error.message())?;
    }

    if bytes_returned < size_of::<STORAGE_DEVICE_DESCRIPTOR>() as u32 {
        return Err("IOCTL_STORAGE_QUERY_PROPERTY returned a short descriptor".to_string());
    }

    output.truncate(bytes_returned as usize);
    Ok(output)
}

fn query_drive_size(handle: HANDLE) -> HwResult<u64> {
    query_drive_length(handle).or_else(|_| query_drive_geometry_size(handle))
}

fn query_drive_length(handle: HANDLE) -> HwResult<u64> {
    let mut output = GET_LENGTH_INFORMATION::default();
    let mut bytes_returned = 0u32;

    unsafe {
        DeviceIoControl(
            handle,
            IOCTL_DISK_GET_LENGTH_INFO,
            None,
            0,
            Some((&mut output as *mut GET_LENGTH_INFORMATION).cast()),
            size_of::<GET_LENGTH_INFORMATION>() as u32,
            Some(&mut bytes_returned),
            None,
        )
        .map_err(|error| error.message())?;
    }

    Ok(output.Length.max(0) as u64)
}

fn query_drive_geometry_size(handle: HANDLE) -> HwResult<u64> {
    let mut output = DISK_GEOMETRY_EX::default();
    let mut bytes_returned = 0u32;

    unsafe {
        DeviceIoControl(
            handle,
            IOCTL_DISK_GET_DRIVE_GEOMETRY_EX,
            None,
            0,
            Some((&mut output as *mut DISK_GEOMETRY_EX).cast()),
            size_of::<DISK_GEOMETRY_EX>() as u32,
            Some(&mut bytes_returned),
            None,
        )
        .map_err(|error| error.message())?;
    }

    Ok(output.DiskSize.max(0) as u64)
}

fn read_descriptor_string<F>(descriptor_bytes: &[u8], offset: F) -> String
where
    F: FnOnce(&STORAGE_DEVICE_DESCRIPTOR) -> u32,
{
    if descriptor_bytes.len() < size_of::<STORAGE_DEVICE_DESCRIPTOR>() {
        return String::new();
    }

    let descriptor = unsafe {
        &*(descriptor_bytes
            .as_ptr()
            .cast::<STORAGE_DEVICE_DESCRIPTOR>())
    };
    let offset = offset(descriptor) as usize;

    if offset == 0 || offset >= descriptor_bytes.len() {
        return String::new();
    }

    let value = &descriptor_bytes[offset..];
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());

    String::from_utf8_lossy(&value[..end]).trim().to_string()
}

pub fn get_logical_storage() -> HwResult<Vec<DiskInfo>> {
    let mut buffer: [u16; 256] = [0; 256];
    let _ = unsafe { GetLogicalDriveStringsW(Some(&mut buffer)) as usize };

    let mut disk_info_list = vec![];
    let drives: Vec<&[u16]> = buffer.split(|&c| c == 0).collect();

    for drive in drives {
        if drive.is_empty() {
            continue;
        }

        let drive_str = String::from_utf16_lossy(drive);
        let drive_cstr = CString::new(drive_str.clone()).map_err(|error| error.to_string())?;
        let drive_path = PCSTR(drive_cstr.as_ptr() as *const u8);

        let mut volume_name = [0u8; 128];
        let mut serial_number: u32 = 0;
        let mut lp_maximum_component_length: u32 = 0;
        let mut file_system_name = [0u8; 128];
        let mut lp_total_number_of_bytes: u64 = 0;

        unsafe {
            let _ = GetVolumeInformationA(
                drive_path,
                Some(&mut volume_name),
                Some(&mut serial_number),
                Some(&mut lp_maximum_component_length),
                None,
                Some(&mut file_system_name),
            );

            let _ =
                GetDiskFreeSpaceExA(drive_path, None, Some(&mut lp_total_number_of_bytes), None);
        }

        disk_info_list.push(DiskInfo {
            name: String::from_utf8_lossy(&volume_name)
                .trim_matches('\0')
                .to_string(),
            model: String::from_utf8_lossy(&file_system_name)
                .trim_matches('\0')
                .to_string(),
            serial_number: serial_number.to_string(),
            size: lp_total_number_of_bytes,
        })
    }

    Ok(disk_info_list)
}

fn wide_null(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

fn is_not_found_message(message: &str) -> bool {
    let lower = message.to_lowercase();
    lower.contains("cannot find")
        || lower.contains("not found")
        || lower.contains("не удается найти")
        || lower.contains("не найден")
}

struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0) };
    }
}
