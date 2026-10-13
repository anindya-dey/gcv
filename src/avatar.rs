//! Best-effort avatar rendering: fetch/load an image and turn it into terminal
//! half-block art. Any failure returns `None` so the caller can fall back.

use crate::ansi::{Rgb, Style};

pub fn render(source: &str, cols: usize) -> Option<Vec<String>> {
    let bytes = load(source)?;
    let img = image::load_from_memory(&bytes).ok()?;
    let (w, h) = (img.width(), img.height());
    if w == 0 || h == 0 {
        return None;
    }

    let cols = cols.max(8);
    let rows = ((cols as f32 * h as f32 / w as f32) / 2.0).round().max(1.0) as u32;
    let px_w = cols as u32;
    let px_h = rows * 2;

    let small = img
        .resize_exact(px_w, px_h, image::imageops::FilterType::Triangle)
        .to_rgba8();

    let mut lines = Vec::with_capacity(rows as usize);
    for r in 0..rows {
        let mut line = String::new();
        for c in 0..cols as u32 {
            let top = small.get_pixel(c, r * 2);
            let bot = if r * 2 + 1 < px_h {
                Some(small.get_pixel(c, r * 2 + 1))
            } else {
                None
            };
            let tc = to_rgb(top);
            match bot {
                Some(b) => {
                    let bc = to_rgb(b);
                    let st = Style::new().fg(tc).bg(bc);
                    line.push_str(&st.paint("▀"));
                }
                None => {
                    line.push_str(&Style::new().fg(tc).paint("▀"));
                }
            }
        }
        lines.push(line);
    }
    Some(lines)
}

fn to_rgb(p: &image::Rgba<u8>) -> Rgb {
    (p[0], p[1], p[2])
}

fn load(source: &str) -> Option<Vec<u8>> {
    if source.starts_with("http://") || source.starts_with("https://") {
        let resp = ureq::get(source)
            .set("User-Agent", concat!("gcv/", env!("CARGO_PKG_VERSION")))
            .call()
            .ok()?;
        let mut reader = resp.into_reader();
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut reader, &mut buf).ok()?;
        Some(buf)
    } else {
        std::fs::read(source).ok()
    }
}
