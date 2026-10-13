//! Themes: named color palettes plus color-name/hex resolution.

use crate::ansi::Rgb;

#[derive(Clone)]
pub struct Theme {
    pub name: &'static str,
    pub fg: Rgb,
    pub heading: Rgb,
    pub accent: Rgb,
    pub accent2: Rgb,
    pub muted: Rgb,
    pub subtle: Rgb,
    pub rule: Rgb,
    pub chip_bg: Rgb,
    pub chip_fg: Rgb,
    pub good: Rgb,
}

impl Theme {
    pub fn get(name: &str) -> Option<Theme> {
        let n = name.to_lowercase();
        let t = match n.as_str() {
            "neon" => Theme {
                name: "neon",
                fg: (226, 232, 240),
                heading: (255, 255, 255),
                accent: (34, 211, 238),
                accent2: (244, 114, 182),
                muted: (148, 163, 184),
                subtle: (100, 116, 139),
                rule: (51, 65, 85),
                chip_bg: (30, 41, 59),
                chip_fg: (125, 211, 252),
                good: (74, 222, 128),
            },
            "dracula" => Theme {
                name: "dracula",
                fg: (248, 248, 242),
                heading: (241, 250, 140),
                accent: (189, 147, 249),
                accent2: (255, 121, 198),
                muted: (98, 114, 164),
                subtle: (68, 71, 90),
                rule: (68, 71, 90),
                chip_bg: (40, 42, 54),
                chip_fg: (139, 233, 253),
                good: (80, 250, 123),
            },
            "nord" => Theme {
                name: "nord",
                fg: (216, 222, 233),
                heading: (236, 239, 244),
                accent: (136, 192, 208),
                accent2: (129, 161, 193),
                muted: (143, 161, 179),
                subtle: (76, 86, 106),
                rule: (59, 66, 82),
                chip_bg: (59, 66, 82),
                chip_fg: (143, 188, 187),
                good: (163, 190, 140),
            },
            "gruvbox" => Theme {
                name: "gruvbox",
                fg: (235, 219, 178),
                heading: (251, 241, 199),
                accent: (250, 189, 47),
                accent2: (254, 128, 25),
                muted: (168, 153, 132),
                subtle: (146, 131, 116),
                rule: (60, 56, 54),
                chip_bg: (60, 56, 54),
                chip_fg: (184, 187, 38),
                good: (184, 187, 38),
            },
            "solarized" => Theme {
                name: "solarized",
                fg: (101, 123, 131),
                heading: (7, 54, 66),
                accent: (38, 139, 210),
                accent2: (42, 161, 152),
                muted: (147, 161, 161),
                subtle: (88, 110, 117),
                rule: (238, 232, 213),
                chip_bg: (238, 232, 213),
                chip_fg: (38, 139, 210),
                good: (133, 153, 0),
            },
            "paper" => Theme {
                name: "paper",
                fg: (36, 41, 47),
                heading: (31, 35, 40),
                accent: (9, 105, 218),
                accent2: (130, 80, 223),
                muted: (87, 96, 106),
                subtle: (140, 149, 159),
                rule: (208, 215, 222),
                chip_bg: (234, 238, 242),
                chip_fg: (9, 105, 218),
                good: (26, 127, 55),
            },
            "ocean" => Theme {
                name: "ocean",
                fg: (192, 202, 245),
                heading: (224, 175, 104),
                accent: (122, 162, 247),
                accent2: (187, 154, 247),
                muted: (86, 95, 137),
                subtle: (65, 72, 104),
                rule: (41, 46, 66),
                chip_bg: (36, 40, 59),
                chip_fg: (125, 207, 255),
                good: (158, 206, 106),
            },
            "matrix" => Theme {
                name: "matrix",
                fg: (182, 247, 193),
                heading: (124, 255, 178),
                accent: (57, 255, 20),
                accent2: (0, 255, 156),
                muted: (47, 107, 58),
                subtle: (35, 78, 44),
                rule: (26, 58, 33),
                chip_bg: (16, 38, 22),
                chip_fg: (57, 255, 20),
                good: (57, 255, 20),
            },
            "mono" => Theme {
                name: "mono",
                fg: (212, 212, 212),
                heading: (255, 255, 255),
                accent: (163, 163, 163),
                accent2: (212, 212, 212),
                muted: (115, 115, 115),
                subtle: (82, 82, 82),
                rule: (64, 64, 64),
                chip_bg: (38, 38, 38),
                chip_fg: (212, 212, 212),
                good: (163, 163, 163),
            },
            _ => return None,
        };
        Some(t)
    }

    pub fn list() -> &'static [&'static str] {
        &[
            "neon",
            "dracula",
            "nord",
            "gruvbox",
            "solarized",
            "paper",
            "ocean",
            "matrix",
            "mono",
        ]
    }

    /// Apply an accent override (color name or #hex) to accent + accent2.
    pub fn with_accent(mut self, spec: &str) -> Theme {
        if let Some(c) = parse_color(spec) {
            self.accent = c;
            self.accent2 = lighten(c, 0.28);
        }
        self
    }
}

pub fn parse_color(spec: &str) -> Option<Rgb> {
    let s = spec.trim().to_lowercase();
    let named = match s.as_str() {
        "black" => Some((30, 30, 30)),
        "red" => Some((248, 81, 73)),
        "green" => Some((63, 185, 80)),
        "yellow" => Some((210, 168, 45)),
        "blue" => Some((88, 166, 255)),
        "magenta" | "purple" => Some((188, 140, 255)),
        "cyan" | "teal" => Some((34, 211, 238)),
        "pink" => Some((244, 114, 182)),
        "orange" => Some((255, 158, 64)),
        "lime" => Some((163, 230, 53)),
        "gold" | "amber" => Some((250, 189, 47)),
        "white" => Some((240, 240, 240)),
        "gray" | "grey" => Some((140, 140, 140)),
        _ => None,
    };
    if named.is_some() {
        return named;
    }
    let hex = s.strip_prefix('#').unwrap_or(&s);
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        return Some((r, g, b));
    }
    if hex.len() == 3 {
        let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
        let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
        let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
        return Some((r, g, b));
    }
    None
}

fn lighten(c: Rgb, t: f32) -> Rgb {
    crate::ansi::lerp(c, (255, 255, 255), t)
}
