# Changelog

## [Unreleased]

## [0.10.0] - 2026-04-24

### Added

- Linux support for motherboard info (via SMBIOS `/sys/firmware/dmi/tables/DMI`)

### Changed

- Motherboard module split into `parser.rs`, `windows.rs`, `linux.rs`

## [0.9.0] - 2026-04-22

### Added

- Linux support for BIOS info (via SMBIOS `/sys/firmware/dmi/tables/DMI`)
- Linux support for memory info (via SMBIOS `/sys/firmware/dmi/tables/DMI`)

### Changed

- BIOS module split into `parser.rs`, `windows.rs`, `linux.rs`

## [0.8.0] - 2026-04-21

### Added

- Linux support for SMBIOS (reads from `/sys/firmware/dmi/tables/DMI`)
- Linux support for OS/platform info (reads from `/etc/os-release` + `uname`)

### Changed

- CPU: replaced `sysinfo` / `num_cpus` with `windows` crate
- All collectors restructured to `*/mod.rs` with separate `tests.rs`
- Renamed project to `hardware-requiem`
- Updated `windows` crate to version 0.62

### Removed

- Dependencies: `sysinfo`, `num_cpus`

## [0.6.1] - 2026-04-17

### Added

- `platform::get_os_info` / `collect_os_info` (Windows)

### Changed

- Moved OS info from `hw` module to `platform` module

## [0.6.0] - 2026-04-17

### Added

- Memory: `serial_number`, `part_number`, `device_locator`, `bank_locator` fields

### Changed

- Memory: replaced `smbios-lib` dependency with internal SMBIOS parser

### Removed

- Dependency: `smbios-lib`

## [0.5.0] - 2026-04-17

### Added

- Motherboard: `asset_tag`, `location_in_chassis`, `board_type`, `feature_flags`,
  `memory_slot_count`, `occupied_memory_slot_count`, `processor_socket` fields
- Internal SMBIOS parser (`hw::smbios`)

### Changed

- Motherboard: replaced WMI with `windows` crate + SMBIOS

## [0.4.0] - 2026-04-17

### Added

- Display info via EDID (Windows)

### Changed

- BIOS: replaced WMI with `windows` crate + SMBIOS

## [0.2.0] - 2026-04-16

### Added

- GPU info via DXGI (Windows)
- Storage info via IOCTL (Windows)
- CPU info (Windows)

## [0.1.0] - 2026-04-16

Initial release based on an internal library from a previous job.
