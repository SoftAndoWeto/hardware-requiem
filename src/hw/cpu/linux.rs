use std::collections::HashSet;

use super::{compose_cpu_identifier, mhz_to_hz, normalize_cpu_brand, CpuInfo, HwResult};

pub fn get_cpu_info() -> HwResult<CpuInfo> {
    let content = std::fs::read_to_string("/proc/cpuinfo")
        .map_err(|e| format!("cannot read /proc/cpuinfo: {e}"))?;
    parse_cpu_info_from_procfs(&content)
}

fn parse_cpu_info_from_procfs(content: &str) -> HwResult<CpuInfo> {
    parse_cpu_info_from_procfs_with_freq(content, read_cpu_max_freq_hz().ok())
}

pub(super) fn parse_cpu_info_from_procfs_with_freq(
    content: &str,
    max_freq_hz: Option<u64>,
) -> HwResult<CpuInfo> {
    let mut name: Option<String> = None;
    let mut vendor_id = String::new();
    let mut cpu_family = String::new();
    let mut model_num = String::new();
    let mut stepping = String::new();
    let mut cpu_mhz: Option<f64> = None;
    let mut cores: HashSet<(String, String)> = HashSet::new();

    for block in content.split("\n\n") {
        let mut physical_id = String::new();
        let mut core_id = String::new();

        for line in block.lines() {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim();

            match key {
                "model name" if name.is_none() => name = Some(normalize_cpu_brand(value)),
                "vendor_id" if vendor_id.is_empty() => vendor_id = value.to_string(),
                "cpu family" if cpu_family.is_empty() => cpu_family = value.to_string(),
                "model" if model_num.is_empty() => model_num = value.to_string(),
                "stepping" if stepping.is_empty() => stepping = value.to_string(),
                "cpu MHz" if cpu_mhz.is_none() => cpu_mhz = value.parse().ok(),
                "physical id" => physical_id = value.to_string(),
                "core id" => core_id = value.to_string(),
                _ => {}
            }
        }

        if !physical_id.is_empty() || !core_id.is_empty() {
            cores.insert((physical_id, core_id));
        }
    }

    let name = name.ok_or("model name not found in /proc/cpuinfo")?;
    let raw_identifier = format!("Family {cpu_family} Model {model_num} Stepping {stepping}");
    let identifier = compose_cpu_identifier(&vendor_id, &raw_identifier);

    let vendor_frequency = max_freq_hz
        .or_else(|| cpu_mhz.map(|mhz| mhz_to_hz(mhz as u64)))
        .ok_or("cannot determine CPU frequency")?;

    let physical_processor_count = if cores.is_empty() {
        content
            .lines()
            .filter(|l| l.starts_with("processor"))
            .count()
            .max(1)
    } else {
        cores.len()
    };

    Ok(CpuInfo {
        name,
        identifier,
        processor_id: None,
        vendor_frequency,
        physical_processor_count,
    })
}

fn read_cpu_max_freq_hz() -> HwResult<u64> {
    let content =
        std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq")
            .map_err(|e| format!("cannot read cpuinfo_max_freq: {e}"))?;
    let khz: u64 = content
        .trim()
        .parse()
        .map_err(|e| format!("cannot parse cpuinfo_max_freq: {e}"))?;
    Ok(khz.saturating_mul(1_000))
}
