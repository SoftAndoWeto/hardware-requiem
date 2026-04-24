use super::edid::{edid_display_name, edid_manufacturer_id, parse_edid_info};

#[test]
fn parses_edid_display_name() {
    let mut edid = vec![0; 128];
    edid[54..72].copy_from_slice(&[
        0x00, 0x00, 0x00, 0xfc, 0x00, b'O', b'd', b'y', b's', b's', b'e', b'y', b' ', b'G', b'5',
        b'2', b'A', 0x0a,
    ]);

    assert_eq!(edid_display_name(&edid), Some("Odyssey G52A".to_string()));
}

#[test]
fn parses_edid_display_name_empty_when_no_descriptor() {
    let edid = vec![0; 128];
    assert_eq!(edid_display_name(&edid), None);
}

#[test]
fn parses_edid_manufacturer_id() {
    // SAM = 0x53 0x41 0x4D → encoded as 5-bit chars in big-endian u16
    // S=19, A=1, M=13 → (19<<10)|(1<<5)|13 = 0x4C2D
    let mut edid = vec![0u8; 16];
    let value: u16 = (19 << 10) | (1 << 5) | 13;
    edid[8] = (value >> 8) as u8;
    edid[9] = (value & 0xff) as u8;

    assert_eq!(edid_manufacturer_id(&edid), Some("SAM".to_string()));
}

#[test]
fn parses_edid_info_manufacture_year() {
    let mut edid = vec![0u8; 128];
    edid[17] = 34; // 1990 + 34 = 2024

    let info = parse_edid_info(&edid);
    assert_eq!(info.manufacture_year, Some(2024));
}

#[test]
fn parses_edid_info_dimensions() {
    let mut edid = vec![0u8; 128];
    edid[21] = 60; // width_cm
    edid[22] = 34; // height_cm

    let info = parse_edid_info(&edid);
    assert_eq!(info.width_cm, Some(60));
    assert_eq!(info.height_cm, Some(34));
    assert!(info.diagonal_inches.is_some());
}

#[test]
#[cfg(windows)]
fn parses_monitor_hardware_id() {
    use super::windows::monitor_hardware_id;

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

#[test]
#[cfg(target_os = "linux")]
fn is_drm_connector_filters_correctly() {
    use super::linux::is_drm_connector;

    assert!(is_drm_connector("card0-DP-1"));
    assert!(is_drm_connector("card2-HDMI-A-1"));
    assert!(is_drm_connector("card10-DP-3"));
    assert!(!is_drm_connector("card0"));
    assert!(!is_drm_connector("card1"));
    assert!(!is_drm_connector("renderD128"));
}
