use super::{DiskInfo, HwResult};

pub fn get_storage() -> HwResult<Vec<DiskInfo>> {
    let mut disks: Vec<DiskInfo> = std::fs::read_dir("/sys/block")
        .map_err(|e| format!("cannot read /sys/block: {e}"))?
        .flatten()
        .filter(|entry| entry.path().join("device").exists())
        .map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            let sys_path = entry.path();
            let model =
                read_sysfs_string(&sys_path.join("device/model")).unwrap_or_default();
            let serial_number =
                read_sysfs_string(&sys_path.join("device/serial")).unwrap_or_default();
            let size = read_sysfs_u64(&sys_path.join("size"))
                .map(sectors_to_bytes)
                .unwrap_or(0);
            DiskInfo {
                name: format!("/dev/{name}"),
                model,
                serial_number,
                size,
            }
        })
        .collect();

    if disks.is_empty() {
        return Err("no physical drives were found".to_string());
    }

    disks.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(disks)
}

fn read_sysfs_string(path: &std::path::Path) -> Option<String> {
    let value = std::fs::read_to_string(path).ok()?;
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() { None } else { Some(trimmed) }
}

fn read_sysfs_u64(path: &std::path::Path) -> Option<u64> {
    read_sysfs_string(path)?.parse().ok()
}

pub(super) fn sectors_to_bytes(sectors: u64) -> u64 {
    sectors.saturating_mul(512)
}
