use pastel::Color;
use regex::Regex;

use crate::x11colors::{NamedColor, X11_COLORS};

fn hex_to_u8_unsafe(hex: &str) -> u8 {
    u8::from_str_radix(hex, 16).unwrap()
}

fn dec_to_u8(hex: &str) -> Option<u8> {
    u8::from_str_radix(hex, 10).ok()
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r, g, b)
}

pub fn parse_color(color: &str) -> Option<Color> {
    let color = color.trim();

    // #RRGGBB
    let re_hex_rrggbb = Regex::new(
        r"(?x)
            ^
            \#?                # optional '#' character
            ([[:xdigit:]]{2})  # two hexadecimal digits (red)
            ([[:xdigit:]]{2})  # two hexadecimal digits (green)
            ([[:xdigit:]]{2})  # two hexadecimal digits (blue)
            $
        ",
    )
    .unwrap();

    if let Some(caps) = re_hex_rrggbb.captures(color) {
        let r = hex_to_u8_unsafe(caps.get(1).unwrap().as_str());
        let g = hex_to_u8_unsafe(caps.get(2).unwrap().as_str());
        let b = hex_to_u8_unsafe(caps.get(3).unwrap().as_str());

        return Some(rgb(r, g, b));
    }

    // #RGB
    let re_hex_rgb = Regex::new(
        r"(?x)
            ^
            \#?             # optional '#' character
            ([[:xdigit:]])  # one hexadecimal digit (red)
            ([[:xdigit:]])  # one hexadecimal digit (green)
            ([[:xdigit:]])  # one hexadecimal digit (blue)
            $
        ",
    )
    .unwrap();

    if let Some(caps) = re_hex_rgb.captures(color) {
        let r = hex_to_u8_unsafe(caps.get(1).unwrap().as_str());
        let g = hex_to_u8_unsafe(caps.get(2).unwrap().as_str());
        let b = hex_to_u8_unsafe(caps.get(3).unwrap().as_str());

        let r = 16 * r + r;
        let g = 16 * g + g;
        let b = 16 * b + b;

        return Some(rgb(r, g, b));
    }

    // rgb(255,0,119)
    let re_hex_rgb = Regex::new(
        r"(?x)
            ^
            rgb\(
                \s*
                ([0-9]{1,3})
                ,
                \s*
                ([0-9]{1,3})
                ,
                \s*
                ([0-9]{1,3})
                \s*
            \)
            $
        ",
    )
    .unwrap();

    if let Some(caps) = re_hex_rgb.captures(color) {
        let mr = dec_to_u8(caps.get(1).unwrap().as_str());
        let mg = dec_to_u8(caps.get(2).unwrap().as_str());
        let mb = dec_to_u8(caps.get(3).unwrap().as_str());

        match (mr, mg, mb) {
            (Some(r), Some(g), Some(b)) => return Some(rgb(r, g, b)),
            _ => {}
        };
    }

    // 255,0,119
    let re_hex_rgb2 = Regex::new(
        r"(?x)
            ^
            ([0-9]{1,3})
            ,
            ([0-9]{1,3})
            ,
            ([0-9]{1,3})
            $
        ",
    )
    .unwrap();

    if let Some(caps) = re_hex_rgb2.captures(color) {
        let mr = dec_to_u8(caps.get(1).unwrap().as_str());
        let mg = dec_to_u8(caps.get(2).unwrap().as_str());
        let mb = dec_to_u8(caps.get(3).unwrap().as_str());

        match (mr, mg, mb) {
            (Some(r), Some(g), Some(b)) => return Some(rgb(r, g, b)),
            _ => {}
        };
    }

    for &NamedColor(name, r, g, b) in X11_COLORS.iter() {
        if color == name {
            return Some(rgb(r, g, b));
        }
    }

    None
}

#[test]
fn parse_hex_rrggbb() {
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
fn parse_hex_rgb() {
    assert_eq!(Some(rgb(255, 0, 119)), parse_color("#f07"));
    assert_eq!(Some(rgb(255, 0, 119)), parse_color("#F07"));
    assert_eq!(Some(rgb(255, 0, 119)), parse_color("f07"));

    assert_eq!(None, parse_color("#h03"));
}

#[test]
fn parse_rgb() {
    assert_eq!(Some(rgb(255, 0, 119)), parse_color("rgb(255,0,119)"));
    assert_eq!(Some(rgb(255, 8, 119)), parse_color("  rgb( 255,  8,  119 )  "));
    assert_eq!(Some(rgb(255, 0, 119)), parse_color("255,0,119"));
    assert_eq!(Some(rgb(1, 2, 3)), parse_color("1,2,3"));

    assert_eq!(None, parse_color("rgb(256,0,0)"));
    assert_eq!(None, parse_color("rgb(255,0)"));
    assert_eq!(None, parse_color("rgb(255,0,0"));
    assert_eq!(None, parse_color("rgb (256,0,0)"));
}
