//! QR code rendering for a shareable link.

use qrcode::render::unicode;
use qrcode::QrCode;

/// Render `data` as plain unicode QR lines (caller styles them).
pub fn render(data: &str) -> Vec<String> {
    let code = match QrCode::new(data.as_bytes()) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let s = code
        .render::<unicode::Dense1x2>()
        .quiet_zone(true)
        .module_dimensions(1, 1)
        .build();
    s.lines().map(|l| l.to_string()).collect()
}
