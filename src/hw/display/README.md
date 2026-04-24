# hw/display

[Документация на русском](README.ru.md)

Collects information about connected display monitors. Supports Windows and Linux.

## Usage

```rust
use hardware_requiem::hw::display::get_display;

let displays = get_display().unwrap();

for display in &displays {
    println!("{}", display.name);

    if let Some(vendor) = &display.vendor {
        println!(" Vendor: {vendor}");
    }
    if let (Some(w), Some(h)) = (display.current_resolution_width, display.current_resolution_height) {
        println!("Resolution: {w}x{h}");
    }
    if let Some(hz) = display.refresh_rate_hz {
        println!("Refresh: {hz} Hz");
    }
    if let Some(inches) = display.diagonal_inches {
        println!("Diagonal: {inches}\"");
    }
}
```

## Windows

Active monitors are enumerated with `EnumDisplayDevicesW`. For each one the
raw EDID binary is read from
`HKLM\SYSTEM\CurrentControlSet\Enum\DISPLAY\<HardwareId>\<Instance>\Device Parameters\EDID`
and parsed. Current resolution, refresh rate, and position come from
`EnumDisplaySettingsW`. If a monitor has no registry EDID entry it is still
included, just without the EDID-derived fields.

## Linux

`/sys/class/drm/` mixes GPU card nodes (`card0`) with connector nodes
(`card0-DP-1`, `card2-HDMI-A-1`). Only connectors with `status == "connected"`
and a valid `edid` file are collected. Resolution and refresh rate are not
available through this path and will be `None`.
