# hw/display

Сбор информации о подключённых мониторах. Поддерживается Windows и Linux.

## Использование

```rust
use hardware_requiem::hw::display::get_display;

let displays = get_display().unwrap();

for display in &displays {
    println!("{}", display.name);

    if let Some(vendor) = &display.vendor {
        println!("Производитель: {vendor}");
    }
    if let (Some(w), Some(h)) = (display.current_resolution_width, display.current_resolution_height) {
        println!("азрешение: {w}x{h}");
    }
    if let Some(hz) = display.refresh_rate_hz {
        println!("Частота: {hz} Гц");
    }
    if let Some(inches) = display.diagonal_inches {
        println!("Диагональ: {inches}\"");
    }
}
```

## Windows

Активные мониторы перечисляются через `EnumDisplayDevicesW`. Для каждого из них
сырой EDID читается из реестра по пути
`HKLM\SYSTEM\CurrentControlSet\Enum\DISPLAY\<HardwareId>\<Instance>\Device Parameters\EDID`
и разбирается. Текущее разрешение, частота обновления и позиция берутся из
`EnumDisplaySettingsW`. Если у монитора нет записи EDID в реестре, он всё равно
попадает в результат - просто без полей, основанных на EDID.

## Linux

В `/sys/class/drm/` перемешаны узлы карт (`card0`) и узлы коннекторов
(`card0-DP-1`, `card2-HDMI-A-1`). Собираются только коннекторы со значением
`status == "connected"` и корректным файлом `edid`. Разрешение и частота
обновления через этот путь недоступны и будут `None`.
