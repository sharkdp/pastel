use crate::colorspace::get_mixing_function;
use crate::commands::prelude::*;

use pastel::ColorblindnessType;
use pastel::Fraction;

fn clamp(lower: f64, upper: f64, x: f64) -> f64 {
    f64::max(f64::min(upper, x), lower)
}

macro_rules! color_command {
    ($cmd_name:ident, $config:ident, $matches:ident, $color:ident, $body:block) => {
        pub struct $cmd_name;

        impl ColorCommand for $cmd_name {
            fn run(
                &self,
                out: &mut Output,
                $matches: &ArgMatches,
                $config: &Config,
                $color: &Color,
            ) -> Result<()> {
                let output = $body;
                out.show_color($config, &output)
            }
        }
    };
}

color_command!(SaturateCommand, _config, matches, color, {
    let amount = number_arg(matches, "amount")?;
    color.saturate(amount)
});

color_command!(DesaturateCommand, _config, matches, color, {
    let amount = number_arg(matches, "amount")?;
    color.desaturate(amount)
});

color_command!(LightenCommand, _config, matches, color, {
    let amount = number_arg(matches, "amount")?;
    color.lighten(amount)
});

color_command!(DarkenCommand, _config, matches, color, {
    let amount = number_arg(matches, "amount")?;
    color.darken(amount)
});

color_command!(RotateCommand, _config, matches, color, {
    let degrees = number_arg(matches, "degrees")?;
    color.rotate_hue(degrees)
});

color_command!(ComplementCommand, _config, _matches, color, {
    color.complementary()
});

color_command!(ToGrayCommand, _config, _matches, color, { color.to_gray() });

color_command!(TextColorCommand, _config, _matches, color, {
    color.text_color()
});

color_command!(MixCommand, config, matches, color, {
    let mut print_spectrum = PrintSpectrum::Yes;

    let base = ColorArgIterator::from_color_arg(
        config,
        matches.value_of("base").expect("required argument"),
        &mut print_spectrum,
    )?;
    let fraction = Fraction::from(1.0 - number_arg(matches, "fraction")?);

    let mix = get_mixing_function(matches.value_of("colorspace").expect("required argument"));

    mix(&base, color, fraction)
});

color_command!(ColorblindCommand, config, matches, color, {
    // The type of colorblindness selected (protanopia, deuteranopia, tritanopia)
    let cb_ty = matches.value_of("type").expect("required argument");
    let cb_ty = cb_ty.to_lowercase();

    // Convert the string to the corresponding enum variant
    let cb_ty = match cb_ty.as_ref() {
        "prot" => ColorblindnessType::Protanopia,
        "deuter" => ColorblindnessType::Deuteranopia,
        "trit" => ColorblindnessType::Tritanopia,
        &_ => {
            unreachable!("Unknown property");
        }
    };

    color.simulate_colorblindness(cb_ty)
});

color_command!(SetCommand, config, matches, color, {
    let property = matches.value_of("property").expect("required argument");
    let property = property.to_lowercase();
    let property = property.as_ref();

    let value = number_arg(matches, "value")?;

    match property {
        "red" | "green" | "blue" => {
            let mut rgba = color.to_rgba();
            let value = clamp(0.0, 255.0, value) as u8;
            match property {
                "red" => {
                    rgba.r = value;
                }
                "green" => {
                    rgba.g = value;
                }
                "blue" => {
                    rgba.b = value;
                }
                _ => unreachable!(),
            }
            Color::from_rgba(rgba.r, rgba.g, rgba.b, rgba.alpha)
        }
        "hsl-hue" | "hsl-saturation" | "hsl-lightness" => {
            let mut hsla = color.to_hsla();
            match property {
                "hsl-hue" => {
                    hsla.h = value;
                }
                "hsl-saturation" => {
                    hsla.s = value;
                }
                "hsl-lightness" => {
                    hsla.l = value;
                }
                _ => unreachable!(),
            }
            Color::from_hsla(hsla.h, hsla.s, hsla.l, hsla.alpha)
        }
        "oklab-l" | "oklab-a" | "oklab-b" => {
            let mut oklab = color.to_oklab();
            match property {
                "oklab-l" => {
                    oklab.l = value;
                }
                "oklab-a" => {
                    oklab.a = value;
                }
                "oklab-b" => {
                    oklab.b = value;
                }
                _ => unreachable!(),
            }
            Color::from_oklab(oklab.l, oklab.a, oklab.b, oklab.alpha)
        }
        "lightness" | "lab-a" | "lab-b" => {
            let mut lab = color.to_lab();
            match property {
                "lightness" => {
                    lab.l = value;
                }
                "lab-a" => {
                    lab.a = value;
                }
                "lab-b" => {
                    lab.b = value;
                }
                _ => unreachable!(),
            }
            Color::from_lab(lab.l, lab.a, lab.b, lab.alpha)
        }
        "hue" | "chroma" => {
            let mut lch = color.to_lch();
            match property {
                "hue" => {
                    lch.h = value;
                }
                "chroma" => {
                    lch.c = value;
                }
                _ => unreachable!(),
            }
            Color::from_lch(lch.l, lch.c, lch.h, lch.alpha)
        }
        "alpha" => {
            let mut hsla = color.to_hsla();
            hsla.alpha = value;
            Color::from_hsla(hsla.h, hsla.s, hsla.l, hsla.alpha)
        }
        &_ => {
            unreachable!("Unknown property");
        }
    }
});
