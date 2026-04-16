use std::{ffi::CString, ffi::OsStr, mem::size_of, os::windows::ffi::OsStrExt};

use serde::{Deserialize, Serialize};
use windows::{
    core::{PCSTR, PCWSTR},
    Win32::Foundation::{CloseHandle, HANDLE},
    Win32::Storage::FileSystem::{
        CreateFileW, GetDiskFreeSpaceExA, GetLogicalDriveStringsW, GetVolumeInformationA,
        FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    },
    Win32::System::{
        Ioctl::{
            PropertyStandardQuery, StorageDeviceProperty, GET_LENGTH_INFORMATION,
            IOCTL_DISK_GET_LENGTH_INFO, IOCTL_STORAGE_QUERY_PROPERTY, STORAGE_DEVICE_DESCRIPTOR,
            STORAGE_PROPERTY_QUERY,
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

/// Retrieves information about all connected storage devices, including hard drives, SSDs, and
/// USB drives.
///
/// # Returns
///
/// A vector of `DiskInfo` structs, each containing detailed information about a storage device.
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
    let wide_path = wide_null(&path);

    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            HANDLE(std::ptr::null_mut()),
        )
    };

    let handle = match handle {
        Ok(handle) => OwnedHandle(handle),
        Err(error) => {
            let message = error.message();
            if message.contains("cannot find") || message.contains("не удается найти")
            {
                return Ok(None);
            }
            return Err(message);
        }
    };

    let descriptor = query_storage_descriptor(handle.0)?;
    let size = query_drive_size(handle.0).unwrap_or_default();

    Ok(Some(DiskInfo {
        name: path,
        model: read_descriptor_string(&descriptor, |descriptor| descriptor.ProductIdOffset),
        serial_number: read_descriptor_string(&descriptor, |descriptor| {
            descriptor.SerialNumberOffset
        }),
        size,
    }))
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
    // Initialize a buffer to hold the drive strings.
    let mut buffer: [u16; 256] = [0; 256];

    // Get the list of logical drives using the Windows API function GetLogicalDriveStringsW.
    let _ = unsafe { GetLogicalDriveStringsW(Some(&mut buffer)) as usize };

    // Split the buffer into individual drive strings.
    let mut disk_info_list = vec![];

    // Initialize an empty vector to store the disk information.
    let drives: Vec<&[u16]> = buffer.split(|&c| c == 0).collect();

    // Iterate over each drive string.
    for drive in drives {
        // Skip empty drive strings.
        if drive.is_empty() {
            continue;
        }

        // Convert the drive string to a Rust string.
        let drive_str = String::from_utf16_lossy(drive);

        // Convert the drive string to a C string.
        let drive_cstr = CString::new(drive_str.clone()).map_err(|error| error.to_string())?;
        let drive_path = PCSTR(drive_cstr.as_ptr() as *const u8);

        // Initialize buffers to hold volume information.
        let mut volume_name = [0u8; 128];
        let mut serial_number: u32 = 0;
        let mut lp_maximum_component_length: u32 = 0;
        let mut file_system_name = [0u8; 128];
        let mut lp_total_number_of_bytes: u64 = 0;

        // Retrieve volume information using the Windows API function GetVolumeInformationA.
        unsafe {
            let _ = GetVolumeInformationA(
                drive_path,
                Some(&mut volume_name),
                Some(&mut serial_number),
                Some(&mut lp_maximum_component_length),
                None,
                Some(&mut file_system_name),
            );

            // Retrieve disk free space using the Windows API function GetDiskFreeSpaceExA.
            let _ =
                GetDiskFreeSpaceExA(drive_path, None, Some(&mut lp_total_number_of_bytes), None);
        }

        // Create a DiskInfo struct and push it to the disk_info_list vector.
        disk_info_list.push(DiskInfo {
            name: String::from_utf8_lossy(&volume_name)
                .trim_matches('\0')
                .to_string(),
            model: String::from_utf8_lossy(&file_system_name)
                .trim_matches('\0')
                .to_string(),
            serial_number: format!("{}", serial_number),
            size: lp_total_number_of_bytes,
        })
    }

    // Return the vector of disk information.
    Ok(disk_info_list)
}

fn wide_null(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0) };
    }
}
