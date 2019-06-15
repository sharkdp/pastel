use palette::Srgb;
use regex::Regex;

use crate::x11colors::{NamedColor, X11_COLORS};

fn hex_to_u8(hex: &str) -> u8 {
    u8::from_str_radix(hex, 16).unwrap()
}

pub fn parse_color(color: &str) -> Option<Srgb<u8>> {
    let color = color.trim();

    // #RRGGBB
    let re_hex_rrggbb =
        Regex::new(r"^#?([[:xdigit:]]{2})([[:xdigit:]]{2})([[:xdigit:]]{2})$").unwrap();

    if let Some(caps) = re_hex_rrggbb.captures(color) {
        let rr = hex_to_u8(caps.get(1).unwrap().as_str());
        let gg = hex_to_u8(caps.get(2).unwrap().as_str());
        let bb = hex_to_u8(caps.get(3).unwrap().as_str());

        return Some(Srgb::from_components((rr, gg, bb)));
    }

    // #RGB
    let re_hex_rgb = Regex::new(r"^#?([[:xdigit:]])([[:xdigit:]])([[:xdigit:]])$").unwrap();

    if let Some(caps) = re_hex_rgb.captures(color) {
        let r = hex_to_u8(caps.get(1).unwrap().as_str());
        let g = hex_to_u8(caps.get(2).unwrap().as_str());
        let b = hex_to_u8(caps.get(3).unwrap().as_str());

        let r = 16 * r + r;
        let g = 16 * g + g;
        let b = 16 * b + b;

        return Some(Srgb::from_components((r, g, b)));
    }

    for &NamedColor(name, r, g, b) in X11_COLORS.iter() {
        if color == name {
            return Some(Srgb::from_components((r, g, b)));
        }
    }

    None
}
