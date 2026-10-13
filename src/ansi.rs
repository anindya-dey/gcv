//! Minimal, dependency-light ANSI styling with automatic color-depth
//! detection and graceful degradation (truecolor -> 256 -> 16 -> none).

use std::io::IsTerminal;
use std::sync::atomic::{AtomicU8, Ordering};

pub type Rgb = (u8, u8, u8);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColorMode {
    TrueColor = 0,
    Ansi256 = 1,
    Ansi16 = 2,
    None = 3,
}

static MODE: AtomicU8 = AtomicU8::new(255);

pub fn set_mode(m: ColorMode) {
    MODE.store(m as u8, Ordering::Relaxed);
}

pub fn mode() -> ColorMode {
    match MODE.load(Ordering::Relaxed) {
        0 => ColorMode::TrueColor,
        1 => ColorMode::Ansi256,
        2 => ColorMode::Ansi16,
        _ => ColorMode::None,
    }
}

/// Decide a color mode from the environment and explicit user intent.
pub fn detect(force_color: bool, force_no_color: bool) -> ColorMode {
    if force_no_color || std::env::var_os("NO_COLOR").is_some() {
        return ColorMode::None;
    }
    let tty = std::io::stdout().is_terminal();
    if !tty && !force_color {
        return ColorMode::None;
    }
    let term = std::env::var("TERM").unwrap_or_default().to_lowercase();
    if term == "dumb" {
        return ColorMode::None;
    }
    let colorterm = std::env::var("COLORTERM")
        .unwrap_or_default()
        .to_lowercase();
    if colorterm.contains("truecolor") || colorterm.contains("24bit") {
        return ColorMode::TrueColor;
    }
    if term.contains("256") {
        return ColorMode::Ansi256;
    }
    if force_color {
        return ColorMode::TrueColor;
    }
    if term.contains("color") {
        return ColorMode::Ansi16;
    }
    ColorMode::TrueColor
}

fn to_256(c: Rgb) -> u8 {
    let (r, g, b) = c;
    if r == g && g == b {
        if r < 8 {
            return 16;
        }
        if r > 248 {
            return 231;
        }
        return 232 + (((r as u16 - 8) * 24) / 247) as u8;
    }
    let ri = (r as u16 * 5 / 255) as u8;
    let gi = (g as u16 * 5 / 255) as u8;
    let bi = (b as u16 * 5 / 255) as u8;
    16 + 36 * ri + 6 * gi + bi
}

const PALETTE16: [Rgb; 16] = [
    (0, 0, 0),
    (205, 49, 49),
    (13, 188, 121),
    (229, 229, 16),
    (36, 114, 200),
    (188, 63, 188),
    (17, 168, 205),
    (229, 229, 229),
    (102, 102, 102),
    (241, 76, 76),
    (35, 209, 139),
    (245, 245, 67),
    (59, 142, 234),
    (214, 112, 214),
    (41, 184, 219),
    (255, 255, 255),
];

fn nearest16(c: Rgb) -> usize {
    let (r, g, b) = (c.0 as i32, c.1 as i32, c.2 as i32);
    let mut best = 0usize;
    let mut best_d = i32::MAX;
    for (i, p) in PALETTE16.iter().enumerate() {
        let dr = r - p.0 as i32;
        let dg = g - p.1 as i32;
        let db = b - p.2 as i32;
        let d = dr * dr + dg * dg + db * db;
        if d < best_d {
            best_d = d;
            best = i;
        }
    }
    best
}

fn fg_code(c: Rgb, m: ColorMode) -> String {
    match m {
        ColorMode::TrueColor => format!("38;2;{};{};{};", c.0, c.1, c.2),
        ColorMode::Ansi256 => format!("38;5;{};", to_256(c)),
        ColorMode::Ansi16 => {
            let idx = nearest16(c);
            let code = if idx < 8 { 30 + idx } else { 90 + idx - 8 };
            format!("{};", code)
        }
        ColorMode::None => String::new(),
    }
}

fn bg_code(c: Rgb, m: ColorMode) -> String {
    match m {
        ColorMode::TrueColor => format!("48;2;{};{};{};", c.0, c.1, c.2),
        ColorMode::Ansi256 => format!("48;5;{};", to_256(c)),
        ColorMode::Ansi16 => {
            let idx = nearest16(c);
            let code = if idx < 8 { 40 + idx } else { 100 + idx - 8 };
            format!("{};", code)
        }
        ColorMode::None => String::new(),
    }
}

#[derive(Clone, Copy, Default)]
pub struct Style {
    pub fg: Option<Rgb>,
    pub bg: Option<Rgb>,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn fg(mut self, c: Rgb) -> Self {
        self.fg = Some(c);
        self
    }
    pub fn bg(mut self, c: Rgb) -> Self {
        self.bg = Some(c);
        self
    }
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    pub fn paint(&self, text: &str) -> String {
        if text.is_empty() {
            return String::new();
        }
        let m = mode();
        if m == ColorMode::None {
            return text.to_string();
        }
        let mut codes = String::new();
        if self.bold {
            codes.push_str("1;");
        }
        if self.dim {
            codes.push_str("2;");
        }
        if self.italic {
            codes.push_str("3;");
        }
        if self.underline {
            codes.push_str("4;");
        }
        if let Some(c) = self.fg {
            codes.push_str(&fg_code(c, m));
        }
        if let Some(c) = self.bg {
            codes.push_str(&bg_code(c, m));
        }
        let codes = codes.trim_end_matches(';');
        if codes.is_empty() {
            return text.to_string();
        }
        format!("\x1b[{}m{}\x1b[0m", codes, text)
    }
}

/// Wrap `text` in an OSC 8 hyperlink so terminals that support it make the
/// text clickable. Falls back to plain text when color is disabled.
pub fn hyperlink(url: &str, text: &str) -> String {
    if mode() == ColorMode::None {
        return text.to_string();
    }
    format!("\x1b]8;;{url}\x1b\\{text}\x1b]8;;\x1b\\")
}

/// Remove ANSI SGR/CSI escape sequences so we can measure true display width.
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                // consume until a final byte in range @..~
                for c in chars.by_ref() {
                    if ('@'..='~').contains(&c) {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}

pub fn vwidth(s: &str) -> usize {
    use unicode_width::UnicodeWidthStr;
    UnicodeWidthStr::width(strip_ansi(s).as_str())
}

/// Visible width of a raw (already un-styled) string.
pub fn twidth(s: &str) -> usize {
    use unicode_width::UnicodeWidthStr;
    UnicodeWidthStr::width(s)
}

/// Interpolate between two colors (t in 0..=1).
pub fn lerp(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    let f = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    (f(a.0, b.0), f(a.1, b.1), f(a.2, b.2))
}
