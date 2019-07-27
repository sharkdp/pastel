use crate::commands::prelude::*;
use pastel::{Fraction, LCh, Lab, HSLA, RGBA};

use super::show::show_color;

macro_rules! color_command {
    ($cmd_name:ident, $matches:ident, $color:ident, $body:block) => {
        pub struct $cmd_name;

        impl ColorCommand for $cmd_name {
            fn run(
                &self,
                out: &mut dyn Write,
                $matches: &ArgMatches,
                config: &Config,
                $color: &Color,
            ) -> Result<()> {
                let output = $body;
                show_color(out, &config, &output)
            }
        }
    };
}

color_command!(SaturateCommand, matches, color, {
    let amount = number_arg(matches, "amount")?;
    color.saturate(amount)
});

color_command!(DesaturateCommand, matches, color, {
    let amount = number_arg(matches, "amount")?;
    color.desaturate(amount)
});

color_command!(LightenCommand, matches, color, {
    let amount = number_arg(matches, "amount")?;
    color.lighten(amount)
});

color_command!(DarkenCommand, matches, color, {
    let amount = number_arg(matches, "amount")?;
    color.darken(amount)
});

color_command!(RotateCommand, matches, color, {
    let degrees = number_arg(matches, "degrees")?;
    color.rotate_hue(degrees)
});

color_command!(ComplementCommand, _matches, color, {
    color.complementary()
});

color_command!(ToGrayCommand, _matches, color, { color.to_gray() });

color_command!(MixCommand, matches, color, {
    let base =
        ColorArgIterator::from_color_arg(matches.value_of("base").expect("required argument"))?;
    let fraction = Fraction::from(1.0 - number_arg(matches, "fraction")?);

    match matches.value_of("colorspace").expect("required argument") {
        "rgb" => base.mix::<RGBA<f64>>(&color, fraction),
        "hsl" => base.mix::<HSLA>(&color, fraction),
        "lab" => base.mix::<Lab>(&color, fraction),
        "lch" => base.mix::<LCh>(&color, fraction),
        _ => unimplemented!("Unknown color space"),
    }
});
