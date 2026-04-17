use super::*;

#[test]
#[cfg(target_os = "windows")]
fn parses_edid_display_name() {
    let mut edid = vec![0; 128];
    edid[54..72].copy_from_slice(&[
        0x00, 0x00, 0x00, 0xfc, 0x00, b'O', b'd', b'y', b's', b's', b'e', b'y', b' ', b'G', b'5',
        b'2', b'A', 0x0a,
    ]);

    assert_eq!(edid_display_name(&edid), Some("Odyssey G52A".to_string()));
}

#[test]
#[cfg(target_os = "windows")]
fn parses_monitor_hardware_id() {
    assert_eq!(
        monitor_hardware_id(r"MONITOR\SAM71E7\{4d36e96e-e325-11ce-bfc1-08002be10318}"),
        Some("SAM71E7".to_string())
    );
    assert_eq!(
        monitor_hardware_id(
            r"\\?\DISPLAY#SAM71E7#5&22264ea&1&UID4353#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}"
        ),
        Some("SAM71E7".to_string())
    );
}
