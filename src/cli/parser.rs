use pastel::Color;
use regex::Regex;

use crate::x11colors::X11_COLORS;

fn hex_to_u8_unsafe(hex: &str) -> u8 {
    u8::from_str_radix(hex, 16).unwrap()
}

fn dec_to_u8(hex: &str) -> Option<u8> {
    u8::from_str_radix(hex, 10).ok()
}

fn dec_to_i32(hex: &str) -> Option<i32> {
    i32::from_str_radix(hex, 10).ok()
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
    let re_rgb = Regex::new(
        r"(?x)
            ^
            rgb\(
                \s*
                ([0-9]{1,3})
                \s*
                ,
                \s*
                ([0-9]{1,3})
                \s*
                ,
                \s*
                ([0-9]{1,3})
                \s*
            \)
            $
        ",
    )
    .unwrap();

    if let Some(caps) = re_rgb.captures(color) {
        let mr = dec_to_u8(caps.get(1).unwrap().as_str());
        let mg = dec_to_u8(caps.get(2).unwrap().as_str());
        let mb = dec_to_u8(caps.get(3).unwrap().as_str());

        match (mr, mg, mb) {
            (Some(r), Some(g), Some(b)) => return Some(rgb(r, g, b)),
            _ => {}
        };
    }

    // RRRGGGBBB without the `rgb(...)` function: 255,0,119
    let re_rgb_nofunction = Regex::new(
        r"(?x)
            ^
            ([0-9]{1,3})
            \s*
            ,
            \s*
            ([0-9]{1,3})
            \s*
            ,
            \s*
            ([0-9]{1,3})
            $
        ",
    )
    .unwrap();

    if let Some(caps) = re_rgb_nofunction.captures(color) {
        let mr = dec_to_u8(caps.get(1).unwrap().as_str());
        let mg = dec_to_u8(caps.get(2).unwrap().as_str());
        let mb = dec_to_u8(caps.get(3).unwrap().as_str());

        match (mr, mg, mb) {
            (Some(r), Some(g), Some(b)) => return Some(rgb(r, g, b)),
            _ => {}
        };
    }

    // hsl(280,35%,40$)
    let re_hsl = Regex::new(
        r"(?x)
            ^
            hsl\(
                \s*
                ([0-9]{1,3})
                \s*
                ,
                \s*
                ([0-9]{1,3})
                %
                \s*
                ,
                \s*
                ([0-9]{1,3})
                %
                \s*
            \)
            $
        ",
    )
    .unwrap();

    if let Some(caps) = re_hsl.captures(color) {
        let mh = dec_to_i32(caps.get(1).unwrap().as_str());
        let ms = dec_to_i32(caps.get(2).unwrap().as_str());
        let ml = dec_to_i32(caps.get(3).unwrap().as_str());

        match (mh, ms, ml) {
            (Some(h), Some(s), Some(l)) => {
                let h = f64::from(h);
                let s = f64::from(s) / 100.0;
                let l = f64::from(l) / 100.0;
                return Some(Color::from_hsl(h, s, l));
            }
            _ => {}
        };
    }

    for nc in X11_COLORS.iter() {
        if color.to_lowercase() == nc.name {
            return Some(nc.color.clone());
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
    assert_eq!(
        Some(rgb(255, 8, 119)),
        parse_color("  rgb( 255  ,  8  ,  119 )  ")
    );
    assert_eq!(Some(rgb(255, 0, 119)), parse_color("255,0,119"));
    assert_eq!(Some(rgb(255, 0, 119)), parse_color("  255  ,  0  ,  119  "));
    assert_eq!(Some(rgb(1, 2, 3)), parse_color("1,2,3"));

    assert_eq!(None, parse_color("rgb(256,0,0)"));
    assert_eq!(None, parse_color("rgb(255,0)"));
    assert_eq!(None, parse_color("rgb(255,0,0"));
    assert_eq!(None, parse_color("rgb (256,0,0)"));
}

#[test]
fn parse_hsl() {
    assert_eq!(
        Some(Color::from_hsl(280.0, 0.2, 0.5)),
        parse_color("hsl(280,20%,50%)")
    );
    assert_eq!(
        Some(Color::from_hsl(280.0, 0.2, 0.5)),
        parse_color("hsl(  280 , 20% , 50%)")
    );

    assert_eq!(None, parse_color("hsl(280,20%,50)"));
    assert_eq!(None, parse_color("hsl(280,20,50%)"));
    assert_eq!(None, parse_color("hsl(280%,20%,50%)"));
    assert_eq!(None, parse_color("hsl(280,20%)"));
}

#[test]
fn parse_predefined_name() {
    assert_eq!(Some(rgb(0, 0, 255)), parse_color("blue"));
    assert_eq!(Some(rgb(0, 0, 255)), parse_color("Blue"));
    assert_eq!(Some(rgb(0, 0, 255)), parse_color("BLUE"));
    assert_eq!(Some(rgb(255, 20, 147)), parse_color("deeppink"));
    assert_eq!(None, parse_color("whatever"));
    assert_eq!(None, parse_color("red blue"));
}
