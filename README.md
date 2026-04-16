# hardware-info

Rust library for collecting basic hardware information.

The current starter implementation targets Windows first and exposes a small
typed API:

```rust
let info = hardware_info::get_hardware_info()?;
println!("{info:#?}");
# Ok::<(), hardware_info::HardwareInfoError>(())
```

Collected on Windows:

- CPU architecture, logical core count, and processor identifier when available
- physical memory totals via `GlobalMemoryStatusEx`
- logical drives and capacity/free space via `GetLogicalDrives`,
  `GetDriveTypeW`, and `GetDiskFreeSpaceExW`
- basic OS family/name fields

Run the example:

```powershell
cargo run --example print
```
