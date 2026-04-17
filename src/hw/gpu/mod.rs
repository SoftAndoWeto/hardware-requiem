use nvml_wrapper::Nvml;
use serde::{Deserialize, Serialize};

use super::HwResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    #[serde(rename = "virtualRam")]
    pub virtual_ram: u64,
    pub vendor: Option<String>,
    #[serde(rename = "vendorId")]
    pub vendor_id: Option<u32>,
    #[serde(rename = "deviceId")]
    pub device_id: Option<u32>,
    #[serde(rename = "dedicatedVideoMemory")]
    pub dedicated_video_memory: Option<u64>,
    #[serde(rename = "dedicatedSystemMemory")]
    pub dedicated_system_memory: Option<u64>,
    #[serde(rename = "sharedSystemMemory")]
    pub shared_system_memory: Option<u64>,
    #[serde(rename = "isSoftware")]
    pub is_software: Option<bool>,
    #[serde(rename = "driverVersion")]
    pub driver_version: Option<String>,
    #[serde(rename = "memoryUsed")]
    pub memory_used: Option<u64>,
    #[serde(rename = "memoryFree")]
    pub memory_free: Option<u64>,
    #[serde(rename = "temperatureCelsius")]
    pub temperature_celsius: Option<u32>,
    #[serde(rename = "utilizationGpuPercent")]
    pub utilization_gpu_percent: Option<u32>,
    #[serde(rename = "utilizationMemoryPercent")]
    pub utilization_memory_percent: Option<u32>,
    #[serde(rename = "powerUsageMilliwatts")]
    pub power_usage_milliwatts: Option<u32>,
}

#[cfg(target_os = "windows")]
pub fn get_gpu() -> HwResult<Vec<GpuInfo>> {
    let mut adapters = collect_dxgi_adapters()?;

    if adapters.is_empty() {
        collect_nvml_gpus()
    } else {
        enrich_with_nvml(&mut adapters);
        Ok(adapters)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_gpu() -> HwResult<Vec<GpuInfo>> {
    collect_nvml_gpus()
}

fn collect_nvml_gpus() -> HwResult<Vec<GpuInfo>> {
    let (driver_version, devices) = collect_nvml_device_info()?;

    Ok(devices
        .into_iter()
        .map(|device| {
            let memory_total = device.memory_total.unwrap_or_default();

            GpuInfo {
                name: device.name,
                virtual_ram: memory_total,
                vendor: Some("NVIDIA".to_string()),
                vendor_id: Some(0x10DE),
                device_id: None,
                dedicated_video_memory: Some(memory_total),
                dedicated_system_memory: None,
                shared_system_memory: None,
                is_software: None,
                driver_version: driver_version.clone(),
                memory_used: device.memory_used,
                memory_free: device.memory_free,
                temperature_celsius: device.temperature_celsius,
                utilization_gpu_percent: device.utilization_gpu_percent,
                utilization_memory_percent: device.utilization_memory_percent,
                power_usage_milliwatts: device.power_usage_milliwatts,
            }
        })
        .collect())
}

fn collect_nvml_device_info() -> HwResult<(Option<String>, Vec<NvmlGpuInfo>)> {
    use nvml_wrapper::enum_wrappers::device::TemperatureSensor;

    let nvml = Nvml::init().map_err(|error| format!("cannot initialize NVML: {error}"))?;

    let device_count = nvml
        .device_count()
        .map_err(|error| format!("cannot get GPU count: {error}"))?;

    let driver_version = nvml.sys_driver_version().ok();
    let mut devices = Vec::with_capacity(device_count as usize);

    for i in 0..device_count {
        let device = nvml
            .device_by_index(i)
            .map_err(|error| format!("cannot access GPU #{i}: {error}"))?;

        let memory = device.memory_info().ok();
        let utilization = device.utilization_rates().ok();

        devices.push(NvmlGpuInfo {
            name: device
                .name()
                .map_err(|error| format!("cannot get GPU #{i} name: {error}"))?,
            memory_total: memory.as_ref().map(|memory| memory.total),
            memory_used: memory.as_ref().map(|memory| memory.used),
            memory_free: memory.as_ref().map(|memory| memory.free),
            temperature_celsius: device.temperature(TemperatureSensor::Gpu).ok(),
            utilization_gpu_percent: utilization.as_ref().map(|utilization| utilization.gpu),
            utilization_memory_percent: utilization.as_ref().map(|utilization| utilization.memory),
            power_usage_milliwatts: device.power_usage().ok(),
        });
    }

    Ok((driver_version, devices))
}

#[derive(Debug)]
struct NvmlGpuInfo {
    name: String,
    memory_total: Option<u64>,
    memory_used: Option<u64>,
    memory_free: Option<u64>,
    temperature_celsius: Option<u32>,
    utilization_gpu_percent: Option<u32>,
    utilization_memory_percent: Option<u32>,
    power_usage_milliwatts: Option<u32>,
}

fn enrich_with_nvml(gpus: &mut [GpuInfo]) {
    let Ok((driver_version, devices)) = collect_nvml_device_info() else {
        return;
    };

    for device in devices {
        let Some(gpu) = find_nvml_target(gpus, &device.name) else {
            continue;
        };

        if let Some(memory_total) = device.memory_total {
            gpu.virtual_ram = memory_total;
            gpu.dedicated_video_memory = Some(memory_total);
        }

        gpu.driver_version = driver_version.clone();
        gpu.memory_used = device.memory_used;
        gpu.memory_free = device.memory_free;
        gpu.temperature_celsius = device.temperature_celsius;
        gpu.utilization_gpu_percent = device.utilization_gpu_percent;
        gpu.utilization_memory_percent = device.utilization_memory_percent;
        gpu.power_usage_milliwatts = device.power_usage_milliwatts;
    }
}

fn find_nvml_target<'a>(gpus: &'a mut [GpuInfo], nvml_name: &str) -> Option<&'a mut GpuInfo> {
    let normalized_nvml_name = normalize_gpu_name(nvml_name);
    let name_match = gpus.iter().position(|gpu| {
        gpu.vendor_id == Some(0x10DE)
            && gpu.driver_version.is_none()
            && normalize_gpu_name(&gpu.name) == normalized_nvml_name
    });

    let fallback_match = || {
        gpus.iter()
            .position(|gpu| gpu.vendor_id == Some(0x10DE) && gpu.driver_version.is_none())
    };

    let index = name_match.or_else(fallback_match)?;
    gpus.get_mut(index)
}

fn normalize_gpu_name(name: &str) -> String {
    name.chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(target_os = "windows")]
fn collect_dxgi_adapters() -> HwResult<Vec<GpuInfo>> {
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE, DXGI_ERROR_NOT_FOUND,
    };

    let factory = unsafe {
        CreateDXGIFactory1::<IDXGIFactory1>()
            .map_err(|error| format!("cannot create DXGI factory: {error}"))?
    };

    let mut gpus = Vec::new();
    let mut index = 0;

    loop {
        let adapter = match unsafe { factory.EnumAdapters1(index) } {
            Ok(adapter) => adapter,
            Err(error) if error.code() == DXGI_ERROR_NOT_FOUND => break,
            Err(error) => return Err(format!("cannot enumerate DXGI adapter #{index}: {error}")),
        };

        let desc = unsafe {
            adapter
                .GetDesc1()
                .map_err(|error| format!("cannot get DXGI adapter #{index} description: {error}"))?
        };

        let dedicated_video_memory = desc.DedicatedVideoMemory as u64;
        let is_software = (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0;

        gpus.push(GpuInfo {
            name: utf16_null_terminated_to_string(&desc.Description),
            virtual_ram: dedicated_video_memory,
            vendor: vendor_name(desc.VendorId).map(str::to_string),
            vendor_id: Some(desc.VendorId),
            device_id: Some(desc.DeviceId),
            dedicated_video_memory: Some(dedicated_video_memory),
            dedicated_system_memory: Some(desc.DedicatedSystemMemory as u64),
            shared_system_memory: Some(desc.SharedSystemMemory as u64),
            is_software: Some(is_software),
            driver_version: None,
            memory_used: None,
            memory_free: None,
            temperature_celsius: None,
            utilization_gpu_percent: None,
            utilization_memory_percent: None,
            power_usage_milliwatts: None,
        });

        index += 1;
    }

    Ok(gpus)
}

#[cfg(target_os = "windows")]
fn utf16_null_terminated_to_string(value: &[u16]) -> String {
    let len = value
        .iter()
        .position(|character| *character == 0)
        .unwrap_or(value.len());

    String::from_utf16_lossy(&value[..len]).trim().to_string()
}

#[cfg(target_os = "windows")]
fn vendor_name(vendor_id: u32) -> Option<&'static str> {
    match vendor_id {
        0x10DE => Some("NVIDIA"),
        0x1002 => Some("AMD"),
        0x8086 => Some("Intel"),
        0x1414 => Some("Microsoft"),
        _ => None,
    }
}
