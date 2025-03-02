use pastel::Color;
use pastel::{Fraction, LCh, Lab, OkLab, HSLA, RGBA};

#[allow(clippy::type_complexity)]
pub fn get_mixing_function(
    colorspace_name: &str,
) -> Box<dyn Fn(&Color, &Color, Fraction) -> Color> {
    match colorspace_name.to_lowercase().as_ref() {
        "rgb" => Box::new(|c1: &Color, c2: &Color, f: Fraction| c1.mix::<RGBA<f64>>(c2, f)),
        "hsl" => Box::new(|c1: &Color, c2: &Color, f: Fraction| c1.mix::<HSLA>(c2, f)),
        "lab" => Box::new(|c1: &Color, c2: &Color, f: Fraction| c1.mix::<Lab>(c2, f)),
        "lch" => Box::new(|c1: &Color, c2: &Color, f: Fraction| c1.mix::<LCh>(c2, f)),
        "oklab" => Box::new(|c1: &Color, c2: &Color, f: Fraction| c1.mix::<OkLab>(c2, f)),
        _ => unreachable!("Unknown color space"),
    }
}
