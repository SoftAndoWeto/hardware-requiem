use super::*;

#[cfg(target_os = "linux")]
use super::linux::parse_cpu_info_from_procfs_with_freq;

#[test]
fn normalizes_cpu_brand_by_trimming_whitespace() {
    let raw = "  Intel(R) Core(TM) i7-14700K  ";
    let normalized = normalize_cpu_brand(raw);
    assert_eq!(normalized, "Intel(R) Core(TM) i7-14700K");
}

#[test]
fn composes_cpu_identifier_from_vendor_and_name() {
    let identifier = compose_cpu_identifier("GenuineIntel", "14th Gen Intel(R) Core(TM)");
    assert_eq!(identifier, "GenuineIntel - 14th Gen Intel(R) Core(TM)");
}

#[test]
fn converts_mhz_to_hz() {
    assert_eq!(mhz_to_hz(3200), 3_200_000_000);
}

#[test]
fn converts_mhz_to_hz_saturates_on_overflow() {
    assert_eq!(mhz_to_hz(u64::MAX), u64::MAX);
}

#[test]
#[cfg(target_os = "linux")]
fn parses_cpu_info_from_procfs() {
    let content = "\
processor\t: 0
vendor_id\t: GenuineIntel
cpu family\t: 6
model\t\t: 183
model name\t:   Intel(R) Core(TM) i7-14700K
stepping\t: 1
cpu MHz\t\t: 3000.000
physical id\t: 0
core id\t\t: 0

processor\t: 1
vendor_id\t: GenuineIntel
cpu family\t: 6
model\t\t: 183
model name\t:   Intel(R) Core(TM) i7-14700K
stepping\t: 1
cpu MHz\t\t: 3000.000
physical id\t: 0
core id\t\t: 1

processor\t: 2
vendor_id\t: GenuineIntel
cpu family\t: 6
model\t\t: 183
model name\t:   Intel(R) Core(TM) i7-14700K
stepping\t: 1
cpu MHz\t\t: 3000.000
physical id\t: 0
core id\t\t: 0

";

    let cpu = parse_cpu_info_from_procfs_with_freq(content, None).unwrap();

    assert_eq!(cpu.name, "Intel(R) Core(TM) i7-14700K");
    assert_eq!(cpu.identifier, "GenuineIntel - Family 6 Model 183 Stepping 1");
    assert_eq!(cpu.vendor_frequency, 3_000_000_000);
    assert_eq!(cpu.physical_processor_count, 2);
    assert!(cpu.processor_id.is_none());
}

#[test]
#[cfg(target_os = "linux")]
fn parses_cpu_info_from_procfs_max_freq_overrides_cpu_mhz() {
    let content = "\
processor\t: 0
vendor_id\t: GenuineIntel
cpu family\t: 6
model\t\t: 183
model name\t: Intel(R) Core(TM) i7-14700K
stepping\t: 1
cpu MHz\t\t: 3000.000
physical id\t: 0
core id\t\t: 0

";

    let cpu = parse_cpu_info_from_procfs_with_freq(content, Some(5_500_000_000)).unwrap();

    assert_eq!(cpu.vendor_frequency, 5_500_000_000);
}

#[test]
#[cfg(target_os = "linux")]
fn parses_cpu_info_from_procfs_without_topology() {
    let content = "\
processor\t: 0
vendor_id\t: AuthenticAMD
cpu family\t: 25
model\t\t: 97
model name\t: AMD Ryzen 9 7950X
stepping\t: 2
cpu MHz\t\t: 4500.000

processor\t: 1
vendor_id\t: AuthenticAMD
cpu family\t: 25
model\t\t: 97
model name\t: AMD Ryzen 9 7950X
stepping\t: 2
cpu MHz\t\t: 4500.000

";

    let cpu = parse_cpu_info_from_procfs_with_freq(content, None).unwrap();

    assert_eq!(cpu.name, "AMD Ryzen 9 7950X");
    assert_eq!(cpu.identifier, "AuthenticAMD - Family 25 Model 97 Stepping 2");
    assert_eq!(cpu.vendor_frequency, 4_500_000_000);
    assert_eq!(cpu.physical_processor_count, 2);
}
