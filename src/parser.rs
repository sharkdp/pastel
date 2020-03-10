use nom::branch::alt;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::error::ErrorKind;
use nom::number::complete::double;
use nom::Err;
use nom::IResult;

use crate::named::NAMED_COLORS;
use crate::Color;

fn hex_to_u8_unsafe(num: &str) -> u8 {
    u8::from_str_radix(num, 16).unwrap()
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r, g, b)
}

fn comma_separated(input: &str) -> IResult<&str, &str> {
    let (input, _) = space0(input)?;
    let (input, _) = char(',')(input)?;
    space0(input)
}

fn parse_separator(input: &str) -> IResult<&str, &str> {
    alt((comma_separated, space1))(input)
}

fn opt_hash_char(s: &str) -> IResult<&str, Option<char>> {
    opt(char('#'))(s)
}

fn parse_percentage(input: &str) -> IResult<&str, f64> {
    let (input, percent) = double(input)?;
    let (input, _) = char('%')(input)?;
    Ok((input, percent / 100.))
}

fn parse_degrees(input: &str) -> IResult<&str, f64> {
    let (input, d) = double(input)?;
    let (input, _) = alt((tag("°"), tag("deg"), tag("")))(input)?;
    Ok((input, d))
}

fn parse_rads(input: &str) -> IResult<&str, f64> {
    let (input, rads) = double(input)?;
    let (input, _) = tag("rad")(input)?;
    Ok((input, rads * 180. / std::f64::consts::PI))
}

fn parse_grads(input: &str) -> IResult<&str, f64> {
    let (input, grads) = double(input)?;
    let (input, _) = tag("grad")(input)?;
    Ok((input, grads * 360. / 400.))
}

fn parse_turns(input: &str) -> IResult<&str, f64> {
    let (input, turns) = double(input)?;
    let (input, _) = tag("turn")(input)?;
    Ok((input, turns * 360.))
}

fn parse_angle(input: &str) -> IResult<&str, f64> {
    alt((parse_turns, parse_grads, parse_rads, parse_degrees))(input)
}

fn parse_hex(input: &str) -> IResult<&str, Color> {
    let (input, _) = opt_hash_char(input)?;
    let (input, hex_chars) = hex_digit1(input)?;
    match hex_chars.len() {
        6 => {
            let r = hex_to_u8_unsafe(&hex_chars[0..2]);
            let g = hex_to_u8_unsafe(&hex_chars[2..4]);
            let b = hex_to_u8_unsafe(&hex_chars[4..6]);
            Ok((input, rgb(r, g, b)))
        }
        3 => {
            let r = hex_to_u8_unsafe(&hex_chars[0..1]);
            let g = hex_to_u8_unsafe(&hex_chars[1..2]);
            let b = hex_to_u8_unsafe(&hex_chars[2..3]);
            let r = r * 16 + r;
            let g = g * 16 + g;
            let b = b * 16 + b;
            Ok((input, rgb(r, g, b)))
        }
        _ => Err(Err::Error((
            "Expected hex string of 3 or 6 characters length",
            ErrorKind::Many1,
        ))),
    }
}

fn parse_numeric_rgb(input: &str) -> IResult<&str, Color> {
    let (input, prefixed) = opt(tag("rgb("))(input)?;
    let is_prefixed = prefixed.is_some();
    let (input, _) = space0(input)?;
    let (input, r) = double(input)?;
    let (input, _) = parse_separator(input)?;
    let (input, g) = double(input)?;
    let (input, _) = parse_separator(input)?;
    let (input, b) = double(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = cond(is_prefixed, char(')'))(input)?;

    let r = r / 255.;
    let g = g / 255.;
    let b = b / 255.;
    let c = Color::from_rgb_float(r, g, b);

    Ok((input, c))
}

fn parse_percentage_rgb(input: &str) -> IResult<&str, Color> {
    let (input, prefixed) = opt(tag("rgb("))(input)?;
    let is_prefixed = prefixed.is_some();
    let (input, _) = space0(input)?;
    let (input, r) = parse_percentage(input)?;
    let (input, _) = parse_separator(input)?;
    let (input, g) = parse_percentage(input)?;
    let (input, _) = parse_separator(input)?;
    let (input, b) = parse_percentage(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = cond(is_prefixed, char(')'))(input)?;

    let c = Color::from_rgb_float(r, g, b);

    Ok((input, c))
}

fn parse_hsl(input: &str) -> IResult<&str, Color> {
    let (input, _) = tag("hsl(")(input)?;
    let (input, _) = space0(input)?;
    let (input, h) = parse_angle(input)?;
    let (input, _) = parse_separator(input)?;
    let (input, s) = parse_percentage(input)?;
    let (input, _) = parse_separator(input)?;
    let (input, l) = parse_percentage(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(')')(input)?;

    let c = Color::from_hsl(h, s, l);

    Ok((input, c))
}

fn parse_gray(input: &str) -> IResult<&str, Color> {
    let (input, _) = tag("gray(")(input)?;
    let (input, _) = space0(input)?;
    let (input, g) = verify(alt((parse_percentage, double)), |&d| d >= 0.)(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(')')(input)?;

    let c = Color::from_rgb_float(g, g, g);

    Ok((input, c))
}

fn parse_lab(input: &str) -> IResult<&str, Color> {
    let (input, _) = opt(tag_no_case("cie"))(input)?;
    let (input, _) = tag_no_case("lab(")(input)?;
    let (input, _) = space0(input)?;
    let (input, l) = double(input)?;
    let (input, _) = parse_separator(input)?;
    let (input, a) = double(input)?;
    let (input, _) = parse_separator(input)?;
    let (input, b) = double(input)?;
    let (input, alpha) = opt(|input: &str| {
        let (input, _) = parse_separator(input)?;
        double(input)
    })(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(')')(input)?;

    let c = Color::from_lab(l, a, b, alpha.unwrap_or(1.0));

    Ok((input, c))
}

fn parse_named(input: &str) -> IResult<&str, Color> {
    let (input, color) = all_consuming(alpha1)(input)?;
    let nc = NAMED_COLORS
        .iter()
        .find(|nc| color.to_lowercase() == nc.name);

    match nc {
        None => Err(Err::Error((
            "Couldn't find matching named color",
            ErrorKind::Alpha,
        ))),
        Some(nc) => Ok((input, nc.color.clone())),
    }
}

pub fn parse_color(input: &str) -> Option<Color> {
    alt((
        all_consuming(parse_hex),
        all_consuming(parse_numeric_rgb),
        all_consuming(parse_percentage_rgb),
        all_consuming(parse_hsl),
        all_consuming(parse_gray),
        all_consuming(parse_lab),
        all_consuming(parse_named),
    ))(input.trim())
    .ok()
    .map(|(_, c)| c)
}

#[test]
fn parse_rgb_hex_syntax() {
    assert_eq!(Some(rgb(255, 0, 153)), parse_color("f09"));
    assert_eq!(Some(rgb(255, 0, 153)), parse_color("#f09"));
    assert_eq!(Some(rgb(255, 0, 153)), parse_color("#F09"));

    assert_eq!(Some(rgb(255, 0, 153)), parse_color("#ff0099"));
    assert_eq!(Some(rgb(255, 0, 153)), parse_color("#FF0099"));
    assert_eq!(Some(rgb(255, 0, 153)), parse_color("ff0099"));

    assert_eq!(Some(rgb(87, 166, 206)), parse_color("57A6CE"));
    assert_eq!(Some(rgb(255, 0, 119)), parse_color("  #ff0077  "));

    assert_eq!(None, parse_color("#1"));
    assert_eq!(None, parse_color("#12"));
    assert_eq!(None, parse_color("#12345"));
    assert_eq!(None, parse_color("#1234567"));
    assert_eq!(None, parse_color("#hh0033"));
    assert_eq!(None, parse_color("#h03"));
}

#[test]
fn parse_rgb_functional_syntax() {
    assert_eq!(Some(rgb(255, 0, 153)), parse_color("rgb(255,0,153)"));
    assert_eq!(Some(rgb(255, 0, 153)), parse_color("rgb(255, 0, 153)"));
    assert_eq!(Some(rgb(255, 0, 153)), parse_color("rgb( 255 , 0 , 153 )"));
    assert_eq!(Some(rgb(255, 0, 153)), parse_color("rgb(255, 0, 153.0)"));
    assert_eq!(Some(rgb(255, 0, 153)), parse_color("rgb(255 0 153)"));

    assert_eq!(
        Some(rgb(255, 8, 119)),
        parse_color("  rgb( 255  ,  8  ,  119 )  ")
    );

    assert_eq!(Some(rgb(255, 0, 127)), parse_color("rgb(100%,0%,49.8%)"));
    assert_eq!(Some(rgb(255, 0, 153)), parse_color("rgb(100%,0%,60%)"));
    assert_eq!(Some(rgb(255, 0, 119)), parse_color("rgb(100%,0%,46.7%)"));
    assert_eq!(Some(rgb(3, 54, 119)), parse_color("rgb(1%,21.2%,46.7%)"));
    assert_eq!(Some(rgb(255, 0, 119)), parse_color("rgb(255 0 119)"));
    assert_eq!(
        Some(rgb(255, 0, 119)),
        parse_color("rgb(    255      0      119)")
    );

    assert_eq!(Some(rgb(255, 0, 153)), parse_color("rgb(100%,0%,60%)"));
    assert_eq!(Some(rgb(255, 0, 153)), parse_color("rgb(100%, 0%, 60%)"));
    assert_eq!(
        Some(rgb(255, 0, 153)),
        parse_color("rgb( 100% , 0% , 60% )")
    );
    assert_eq!(Some(rgb(255, 0, 153)), parse_color("rgb(100% 0% 60%)"));

    assert_eq!(Some(rgb(100, 5, 1)), parse_color("rgb(1e2, .5e1, .5e0)"));
    assert_eq!(Some(rgb(140, 0, 153)), parse_color("rgb(55% 0% 60%)"));
    assert_eq!(Some(rgb(142, 0, 153)), parse_color("rgb(55.5% 0% 60%)"));
    assert_eq!(Some(rgb(255, 0, 0)), parse_color("rgb(256,0,0)"));
    assert_eq!(Some(rgb(255, 255, 0)), parse_color("rgb(100%,100%,-45%)"));

    assert_eq!(None, parse_color("rgb(255,0)"));
    assert_eq!(None, parse_color("rgb(255,0,0"));
    assert_eq!(None, parse_color("rgb (256,0,0)"));
    assert_eq!(None, parse_color("rgb(100%,0,0)"));
    assert_eq!(None, parse_color("rgb(2550119)"));
}

#[test]
fn parse_rgb_standalone_syntax() {
    assert_eq!(
        Some(rgb(255, 8, 119)),
        parse_color("  rgb( 255  ,  8  ,  119 )  ")
    );

    assert_eq!(rgb(255, 0, 153), parse_color("255,0,153").unwrap());
    assert_eq!(rgb(255, 0, 153), parse_color("255, 0, 153").unwrap());
    assert_eq!(
        rgb(255, 0, 153),
        parse_color("  255  ,  0  ,  153   ").unwrap()
    );
    assert_eq!(rgb(255, 0, 153), parse_color("255 0 153").unwrap());
    assert_eq!(rgb(255, 0, 153), parse_color("255 0 153.0").unwrap());

    assert_eq!(Some(rgb(1, 2, 3)), parse_color("1,2,3"));
}

#[test]
fn parse_hsl_syntax() {
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
        Some(Color::from_hsl(270.0, 0.6, 0.7)),
        parse_color("hsl(270 60% 70%)")
    );

    assert_eq!(
        Some(Color::from_hsl(-140.0, 0.2, 0.5)),
        parse_color("hsl(-140°,20%,50%)")
    );

    assert_eq!(
        Some(Color::from_hsl(90.0, 0.2, 0.5)),
        parse_color("hsl(100grad,20%,50%)")
    );
    assert_eq!(
        Some(Color::from_hsl(90.05, 0.2, 0.5)),
        parse_color("hsl(1.5708rad,20%,50%)")
    );
    assert_eq!(
        Some(Color::from_hsl(90.0, 0.2, 0.5)),
        parse_color("hsl(0.25turn,20%,50%)")
    );
    assert_eq!(
        Some(Color::from_hsl(45.0, 0.2, 0.5)),
        parse_color("hsl(50grad,20%,50%)")
    );
    assert_eq!(
        Some(Color::from_hsl(45.0, 0.2, 0.5)),
        parse_color("hsl(0.7854rad,20%,50%)")
    );
    assert_eq!(
        Some(Color::from_hsl(45.0, 0.2, 0.5)),
        parse_color("hsl(0.125turn,20%,50%)")
    );

    assert_eq!(None, parse_color("hsl(280,20%,50)"));
    assert_eq!(None, parse_color("hsl(280,20,50%)"));
    assert_eq!(None, parse_color("hsl(280%,20%,50%)"));
    assert_eq!(None, parse_color("hsl(280,20%)"));
}

#[test]
fn parse_gray_syntax() {
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

    assert_eq!(Some(Color::graytone(0.2)), parse_color("gray(20%)"));
    assert_eq!(Some(Color::black()), parse_color("gray(0%)"));
    assert_eq!(Some(Color::black()), parse_color("gray(0.0%)"));
    assert_eq!(Some(Color::white()), parse_color("gray(100%)"));
    assert_eq!(Some(Color::graytone(0.5)), parse_color("gray(50%)"));

    assert_eq!(None, parse_color("gray(-1)"));
    assert_eq!(None, parse_color("gray(-1%)"));
    assert_eq!(None, parse_color("gray(-4.%)"));
}

#[test]
fn parse_lab_syntax() {
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
fn parse_named_syntax() {
    assert_eq!(Some(Color::black()), parse_color("black"));
    assert_eq!(Some(Color::blue()), parse_color("blue"));
    assert_eq!(Some(Color::blue()), parse_color("Blue"));
    assert_eq!(Some(Color::blue()), parse_color("BLUE"));
    assert_eq!(Some(rgb(255, 20, 147)), parse_color("deeppink"));
    assert_eq!(None, parse_color("whatever"));
    assert_eq!(None, parse_color("red blue"));
}
