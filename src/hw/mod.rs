//! Legacy hardware collectors from the first prototype.
//!
//! This module keeps the DTO-shaped API that was already started and makes it
//! usable from a plain Rust library, without Tauri commands or app-specific
//! dependencies.

pub mod bios;
pub mod cpu;
pub mod display;
pub mod gpu;
pub mod memory;
pub mod motherboard;
mod smbios;
pub mod storage;

use serde::{Deserialize, Serialize};

pub use bios::{get_bios_info, BiosInfo};
pub use cpu::{get_cpu_info, CpuInfo};
pub use display::{get_display, DisplayInfo};
pub use gpu::{get_gpu, GpuInfo};
pub use memory::{get_memory_info, MemoryInfo};
pub use motherboard::{get_motherboard_info, MotherboardInfo};
pub use storage::{get_storage, DiskInfo};

pub type HwResult<T> = std::result::Result<T, String>;

#[derive(Debug, Serialize, Deserialize)]
pub struct HardWareInfo {
    pub bios: Option<BiosInfo>,
    pub motherboard: Option<MotherboardInfo>,
    pub cpu: Option<CpuInfo>,
    pub memory: Vec<MemoryInfo>,
    pub storage: Vec<DiskInfo>,
    pub gpu: Vec<GpuInfo>,
    pub display: Vec<DisplayInfo>,
    pub errors: Vec<CollectionError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionError {
    pub collector: String,
    pub message: String,
}

#[cfg(target_os = "windows")]
pub fn get_hw_info() -> HardWareInfo {
    let mut errors = Vec::new();

    let bios = collect_optional("bios", get_bios_info(), &mut errors);
    let motherboard = collect_optional("motherboard", get_motherboard_info(), &mut errors);
    let cpu = collect_optional("cpu", get_cpu_info(), &mut errors);
    let memory = collect_list("memory", get_memory_info(), &mut errors);
    let storage = collect_list("storage", get_storage(), &mut errors);
    let gpu = collect_list("gpu", get_gpu(), &mut errors);
    let display = collect_list("display", get_display(), &mut errors);

    HardWareInfo {
        bios,
        motherboard,
        cpu,
        memory,
        storage,
        gpu,
        display,
        errors,
    }
}

fn collect_optional<T>(
    collector: &'static str,
    result: HwResult<T>,
    errors: &mut Vec<CollectionError>,
) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(message) => {
            errors.push(CollectionError {
                collector: collector.to_string(),
                message,
            });
            None
        }
    }
}

fn collect_list<T>(
    collector: &'static str,
    result: HwResult<Vec<T>>,
    errors: &mut Vec<CollectionError>,
) -> Vec<T> {
    match result {
        Ok(value) => value,
        Err(message) => {
            errors.push(CollectionError {
                collector: collector.to_string(),
                message,
            });
            Vec::new()
        }
    }
}
