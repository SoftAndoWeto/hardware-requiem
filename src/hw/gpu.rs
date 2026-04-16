use nvml_wrapper::Nvml;
use serde::{Deserialize, Serialize};

use super::HwResult;

/// Represents information about a GPU (Graphics Processing Unit).
///
/// This struct is used to store and serialize data related to an individual GPU in the system.
/// It contains the following fields:
/// - name: A string representing the name of the GPU.
/// - virtual_ram: An unsigned 64-bit integer representing the total amount of virtual RAM
///   available on the GPU, measured in bytes. This value indicates the memory capacity that the
///   GPU can utilize for processing tasks.
#[derive(Debug, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    #[serde(rename = "virtualRam")]
    pub virtual_ram: u64,
}

/// Retrieves information about the available GPUs.
///
/// This function uses the NVML (NVIDIA Management Library) to gather information about the GPUs.
/// It initializes NVML, retrieves the count of available GPUs, and then iterates through each GPU,
/// collecting relevant information such as the name and total virtual RAM.
///
/// # Returns
///
/// A vector of `GpuInfo` structs, where each struct contains information about a single GPU.
pub fn get_gpu() -> HwResult<Vec<GpuInfo>> {
    // Initialize NVML
    let nvml = Nvml::init().map_err(|error| format!("cannot initialize NVML: {error}"))?;

    // Get the count of available GPUs
    let device_count = nvml
        .device_count()
        .map_err(|error| format!("cannot get GPU count: {error}"))?;

    // Create a vector to store the GPU information
    let mut gpu_info_list = vec![];

    // Iterate through each GPU
    for i in 0..device_count {
        // Get the device by index
        let device = nvml
            .device_by_index(i)
            .map_err(|error| format!("cannot access GPU #{i}: {error}"))?;

        // Collect GPU information and push it to the vector
        gpu_info_list.push(GpuInfo {
            name: device
                .name()
                .map_err(|error| format!("cannot get GPU #{i} name: {error}"))?,
            virtual_ram: device
                .memory_info()
                .map_err(|error| format!("cannot get GPU #{i} memory info: {error}"))?
                .total,
        })
    }

    // Return the vector of GPU information
    Ok(gpu_info_list)
}
