use lazy_static::lazy_static;
use pastel::Color;
use regex::Regex;

use crate::named::NAMED_COLORS;

fn hex_to_u8_unsafe(num: &str) -> u8 {
    u8::from_str_radix(num, 16).unwrap()
}

fn dec_to_u8(num: &str) -> Option<u8> {
    u8::from_str_radix(num, 10).ok()
}

fn float_to_f64(num: &str) -> Option<f64> {
    num.parse::<f64>().ok()
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r, g, b)
}

lazy_static! {
    // #RRGGBB
    pub static ref RE_HEX_RRGGBB: Regex = Regex::new(
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

    // #RGB
    pub static ref RE_HEX_RGB: Regex = Regex::new(
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

    // rgb(255,0,119)
    pub static ref RE_RGB: Regex = Regex::new(
        r"(?x)
            ^
            rgb\(
                \s*
                (\d{1,3})
                \s*
                ,
                \s*
                (\d{1,3})
                \s*
                ,
                \s*
                (\d{1,3})
                \s*
            \)
            $
        ",
    )
    .unwrap();

    // RRRGGGBBB without the `rgb(...)` function: 255,0,119
    pub static ref RE_RGB_NOFUNCTION: Regex = Regex::new(
        r"(?x)
            ^
            (\d{1,3})
            \s*
            ,
            \s*
            (\d{1,3})
            \s*
            ,
            \s*
            (\d{1,3})
            $
        ",
    )
    .unwrap();

    // hsl(280,35%,40$)
    pub static ref RE_HSL: Regex = Regex::new(
        r"(?x)
            ^
            hsl\(
                \s*
                (-?\d+(?:\.\d+)?)
                (?:deg|°)?
                \s*
                ,
                \s*
                (\d+(?:\.\d+)?)
                %
                \s*
                ,
                \s*
                (\d+(?:\.\d+)?)
                %
                \s*
            \)
            $
        ",
    )
    .unwrap();

    // gray(0.2)
    pub static ref RE_GRAY: Regex = Regex::new(
        r"(?x)
            ^
            gray\(
            \s*
            (
                    \d+(\.\d+)?   # 1 or 0.321
                |
                    \.\d+         # .23
            )
            \s*
            \)
        ",
    )
    .unwrap();

    // Lab(53.2,-35.4,-68.12,0.5)
    pub static ref RE_LAB: Regex = Regex::new(
    r"(?ix)
        ^
        (?:cie)?lab\(
            \s*
            (\d+(?:\.\d+)?) # L value
            \s*
            ,
            \s*
            (-?\d+(?:\.\d+)?) # A value
            \s*
            ,
            \s*
            (-?\d+(?:\.\d+)?) # B value
            \s*
            (?:
            ,
            \s*
            (\d+(?:\.\d+)?)
            \s*
            )?
        \)$
    ")
    .unwrap();
}

pub fn parse_color(color: &str) -> Option<Color> {
    let color = color.trim();

    if let Some(caps) = RE_HEX_RRGGBB.captures(color) {
        let r = hex_to_u8_unsafe(caps.get(1).unwrap().as_str());
        let g = hex_to_u8_unsafe(caps.get(2).unwrap().as_str());
        let b = hex_to_u8_unsafe(caps.get(3).unwrap().as_str());

        return Some(rgb(r, g, b));
    }

    if let Some(caps) = RE_HEX_RGB.captures(color) {
        let r = hex_to_u8_unsafe(caps.get(1).unwrap().as_str());
        let g = hex_to_u8_unsafe(caps.get(2).unwrap().as_str());
        let b = hex_to_u8_unsafe(caps.get(3).unwrap().as_str());

        let r = 16 * r + r;
        let g = 16 * g + g;
        let b = 16 * b + b;

        return Some(rgb(r, g, b));
    }

    if let Some(caps) = RE_RGB.captures(color) {
        let mr = dec_to_u8(caps.get(1).unwrap().as_str());
        let mg = dec_to_u8(caps.get(2).unwrap().as_str());
        let mb = dec_to_u8(caps.get(3).unwrap().as_str());

        match (mr, mg, mb) {
            (Some(r), Some(g), Some(b)) => return Some(rgb(r, g, b)),
            _ => {}
        };
    }

    if let Some(caps) = RE_RGB_NOFUNCTION.captures(color) {
        let mr = dec_to_u8(caps.get(1).unwrap().as_str());
        let mg = dec_to_u8(caps.get(2).unwrap().as_str());
        let mb = dec_to_u8(caps.get(3).unwrap().as_str());

        match (mr, mg, mb) {
            (Some(r), Some(g), Some(b)) => return Some(rgb(r, g, b)),
            _ => {}
        };
    }

    if let Some(caps) = RE_HSL.captures(color) {
        let mh = float_to_f64(caps.get(1).unwrap().as_str());
        let ms = float_to_f64(caps.get(2).unwrap().as_str());
        let ml = float_to_f64(caps.get(3).unwrap().as_str());

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

    if let Some(caps) = RE_GRAY.captures(color) {
        if let Some(lightness) = float_to_f64(caps.get(1).unwrap().as_str()) {
            return Some(Color::graytone(lightness));
        }
    }

    if let Some(caps) = RE_LAB.captures(color) {
        let ml = float_to_f64(caps.get(1).unwrap().as_str());
        let ma = float_to_f64(caps.get(2).unwrap().as_str());
        let mb = float_to_f64(caps.get(3).unwrap().as_str());
        let malpha = match caps.get(4) {
            Some(a) => float_to_f64(a.as_str()),
            _ => Some(1.0),
        };

        match (ml, ma, mb, malpha) {
            (Some(l), Some(a), Some(b), Some(alpha)) => {
                let l = f64::from(l);
                let a = f64::from(a);
                let b = f64::from(b);
                let alpha = f64::from(alpha);

                return Some(Color::from_lab(l, a, b, alpha));
            }
            _ => {}
        }
    }

    for nc in NAMED_COLORS.iter() {
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
        parse_color("hsl(280deg,20%,50%)")
    );
    assert_eq!(
        Some(Color::from_hsl(280.0, 0.2, 0.5)),
        parse_color("hsl(280°,20%,50%)")
    );
    assert_eq!(
        Some(Color::from_hsl(280.33, 0.123, 0.456)),
        parse_color("hsl(280.33001,12.3%,45.6%)")
    );
    assert_eq!(
        Some(Color::from_hsl(280.0, 0.2, 0.5)),
        parse_color("hsl(  280 , 20% , 50%)")
    );

    assert_eq!(
        Some(Color::from_hsl(-140.0, 0.2, 0.5)),
        parse_color("hsl(-140°,20%,50%)")
    );

    assert_eq!(None, parse_color("hsl(280,20%,50)"));
    assert_eq!(None, parse_color("hsl(280,20,50%)"));
    assert_eq!(None, parse_color("hsl(280%,20%,50%)"));
    assert_eq!(None, parse_color("hsl(280,20%)"));
}

#[test]
fn parse_gray() {
    assert_eq!(Some(Color::graytone(0.2)), parse_color("gray(0.2)"));
    assert_eq!(Some(Color::black()), parse_color("gray(0.0)"));
    assert_eq!(Some(Color::black()), parse_color("gray(0)"));
    assert_eq!(Some(Color::white()), parse_color("gray(1.0)"));
    assert_eq!(Some(Color::white()), parse_color("gray(1)"));
    assert_eq!(Some(Color::white()), parse_color("gray(7.3)"));

    assert_eq!(Some(Color::graytone(0.32)), parse_color("gray(.32)"));

    assert_eq!(
        Some(Color::graytone(0.41)),
        parse_color("  gray(  0.41   ) ")
    );

    assert_eq!(None, parse_color("gray(1.)"));
    assert_eq!(None, parse_color("gray(-1)"));
}

#[test]
fn parse_lab() {
    assert_eq!(
        Some(Color::from_lab(12.43, -35.5, 43.4, 1.0)),
        parse_color("Lab(12.43,-35.5,43.4)")
    );
    assert_eq!(
        Some(Color::from_lab(15.0, -23.0, 43.0, 0.5)),
        parse_color("lab(15,-23,43,0.5)")
    );
    assert_eq!(
        Some(Color::from_lab(15.0, 23.0, -43.0, 1.0)),
        parse_color("CIELab(15,23,-43)")
    );
    assert_eq!(
        Some(Color::from_lab(15.0, 35.5, -43.4, 1.0)),
        parse_color("CIELab(15,35.5,-43.4)")
    );
    assert_eq!(
        Some(Color::from_lab(15.0, -35.5, -43.4, 0.4)),
        parse_color("cieLab(15,-35.5,-43.4,0.4)")
    );
    assert_eq!(
        Some(Color::from_lab(15.0, 23.0, -43.0, 1.0)),
        parse_color("Lab(        15,  23,-43   )")
    );
    assert_eq!(
        Some(Color::from_lab(15.0, -35.5, -43.4, 0.4)),
        parse_color("CieLab(15,-35.5,-43.4,0.4)")
    );
    assert_eq!(
        Some(Color::from_lab(15.0, 23.0, -43.0, 1.0)),
        parse_color("CIELab(        15,  23,-43   )")
    );
}

#[test]
fn parse_predefined_name() {
    assert_eq!(Some(Color::black()), parse_color("black"));
    assert_eq!(Some(Color::blue()), parse_color("blue"));
    assert_eq!(Some(Color::blue()), parse_color("Blue"));
    assert_eq!(Some(Color::blue()), parse_color("BLUE"));
    assert_eq!(Some(rgb(255, 20, 147)), parse_color("deeppink"));
    assert_eq!(None, parse_color("whatever"));
    assert_eq!(None, parse_color("red blue"));
}
