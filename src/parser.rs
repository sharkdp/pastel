use palette::Srgb;
use regex::Regex;

use crate::x11colors::{NamedColor, X11_COLORS};

fn hex_to_u8(hex: &str) -> u8 {
    u8::from_str_radix(hex, 16).unwrap()
}

fn rgb(r: u8, g: u8, b: u8) -> Srgb<u8> {
    Srgb::from_components((r, g, b))
}

pub fn parse_color(color: &str) -> Option<Srgb<u8>> {
    let color = color.trim();

    // #RRGGBB
    let re_hex_rrggbb =
        Regex::new(r"^#?([[:xdigit:]]{2})([[:xdigit:]]{2})([[:xdigit:]]{2})$").unwrap();

    if let Some(caps) = re_hex_rrggbb.captures(color) {
        let r = hex_to_u8(caps.get(1).unwrap().as_str());
        let g = hex_to_u8(caps.get(2).unwrap().as_str());
        let b = hex_to_u8(caps.get(3).unwrap().as_str());

        return Some(rgb(r, g, b));
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

        return Some(rgb(r, g, b));
    }

    for &NamedColor(name, r, g, b) in X11_COLORS.iter() {
        if color == name {
            return Some(rgb(r, g, b));
        }
    }

    None
}

#[test]
fn parse_rrggbb() {
    assert_eq!(Some(rgb(255, 0, 119)), parse_color("#ff0077"));
    assert_eq!(Some(rgb(255, 0, 119)), parse_color("#FF0077"));
    assert_eq!(Some(rgb(255, 0, 119)), parse_color("ff0077"));
    assert_eq!(Some(rgb(87, 166, 206)), parse_color("57A6CE"));
    assert_eq!(Some(rgb(255, 0, 119)), parse_color("  #ff0077  "));

    assert_eq!(None, parse_color("#1"));
    assert_eq!(None, parse_color("#12345"));
    assert_eq!(None, parse_color("#1234567"));
    assert_eq!(None, parse_color("#hh0033"));
}

#[test]
fn parse_rgb() {
    assert_eq!(Some(rgb(255, 0, 119)), parse_color("#f07"));
    assert_eq!(Some(rgb(255, 0, 119)), parse_color("#F07"));
    assert_eq!(Some(rgb(255, 0, 119)), parse_color("f07"));

    assert_eq!(None, parse_color("#h03"));
}
