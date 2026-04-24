use super::*;

#[test]
#[cfg(windows)]
fn reads_descriptor_string_from_valid_offset() {
    let descriptor = descriptor_bytes(&[(64, b"Samsung SSD 990 PRO 1TB\0")], |descriptor| {
        descriptor.ProductIdOffset = 64;
    });

    assert_eq!(
        read_descriptor_string(&descriptor, |descriptor| descriptor.ProductIdOffset),
        "Samsung SSD 990 PRO 1TB"
    );
}

#[test]
#[cfg(windows)]
fn trims_descriptor_string_whitespace() {
    let descriptor = descriptor_bytes(&[(64, b"  S6Z2NJ0W123456A  \0")], |descriptor| {
        descriptor.SerialNumberOffset = 64;
    });

    assert_eq!(
        read_descriptor_string(&descriptor, |descriptor| descriptor.SerialNumberOffset),
        "S6Z2NJ0W123456A"
    );
}

#[test]
#[cfg(windows)]
fn reads_descriptor_string_until_buffer_end_when_missing_null_terminator() {
    let descriptor = descriptor_bytes(&[(64, b"ST2000DM005-2U9102")], |descriptor| {
        descriptor.ProductIdOffset = 64;
    });

    assert_eq!(
        read_descriptor_string(&descriptor, |descriptor| descriptor.ProductIdOffset),
        "ST2000DM005-2U9102"
    );
}

#[test]
#[cfg(windows)]
fn returns_empty_descriptor_string_for_short_descriptor() {
    assert_eq!(
        read_descriptor_string(&[0; 4], |descriptor| descriptor.ProductIdOffset),
        ""
    );
}

#[test]
#[cfg(windows)]
fn returns_empty_descriptor_string_for_zero_offset() {
    let descriptor = descriptor_bytes(&[(64, b"ignored\0")], |descriptor| {
        descriptor.ProductIdOffset = 0;
    });

    assert_eq!(
        read_descriptor_string(&descriptor, |descriptor| descriptor.ProductIdOffset),
        ""
    );
}

#[test]
#[cfg(windows)]
fn returns_empty_descriptor_string_for_out_of_bounds_offset() {
    let descriptor = descriptor_bytes(&[(64, b"ignored\0")], |descriptor| {
        descriptor.ProductIdOffset = 512;
    });

    assert_eq!(
        read_descriptor_string(&descriptor, |descriptor| descriptor.ProductIdOffset),
        ""
    );
}

#[test]
#[cfg(windows)]
fn builds_null_terminated_utf16_paths() {
    assert_eq!(
        wide_null(r"\\.\PhysicalDrive0"),
        r"\\.\PhysicalDrive0"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>()
    );
}

#[test]
#[cfg(windows)]
fn recognizes_not_found_messages() {
    assert!(is_not_found_message(
        "The system cannot find the file specified."
    ));
    assert!(is_not_found_message("device not found"));
    assert!(is_not_found_message("Не удается найти указанный файл."));
    assert!(is_not_found_message("Устройство не найдено."));
    assert!(!is_not_found_message("Access is denied."));
}

#[cfg(windows)]
fn descriptor_bytes<F>(strings: &[(usize, &[u8])], configure: F) -> Vec<u8>
where
    F: FnOnce(&mut STORAGE_DEVICE_DESCRIPTOR),
{
    let mut descriptor = STORAGE_DEVICE_DESCRIPTOR {
        Version: size_of::<STORAGE_DEVICE_DESCRIPTOR>() as u32,
        Size: 128,
        ..Default::default()
    };
    configure(&mut descriptor);

    let mut bytes = vec![0; 128];
    let descriptor_bytes = unsafe {
        std::slice::from_raw_parts(
            (&descriptor as *const STORAGE_DEVICE_DESCRIPTOR).cast::<u8>(),
            size_of::<STORAGE_DEVICE_DESCRIPTOR>(),
        )
    };
    bytes[..descriptor_bytes.len()].copy_from_slice(descriptor_bytes);

    for (offset, value) in strings {
        let end = offset + value.len();
        bytes[*offset..end].copy_from_slice(value);
    }

    bytes
}

#[test]
#[cfg(target_os = "linux")]
fn converts_sectors_to_bytes() {
    assert_eq!(sectors_to_bytes(1953525168), 1_000_204_886_016);
}

#[test]
#[cfg(target_os = "linux")]
fn converts_sectors_to_bytes_saturates_on_overflow() {
    assert_eq!(sectors_to_bytes(u64::MAX), u64::MAX);
}
