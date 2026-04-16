#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardwareInfo {
    pub os: OsInfo,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsInfo {
    pub family: String,
    pub name: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuInfo {
    pub architecture: CpuArchitecture,
    pub logical_cores: usize,
    pub vendor_or_brand: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuArchitecture {
    X86,
    X86_64,
    Arm,
    Aarch64,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryInfo {
    pub total_physical_bytes: u64,
    pub available_physical_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskInfo {
    pub mount_point: String,
    pub kind: DiskKind,
    pub total_bytes: u64,
    pub free_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskKind {
    Fixed,
    Removable,
    Network,
    Optical,
    RamDisk,
    Unknown,
}
