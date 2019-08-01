use crate::commands::prelude::*;
use crate::commands::show::show_color;

use pastel::{Fraction, LCh, Lab, HSLA, RGBA};

pub struct ScaleCommand;

impl GenericCommand for ScaleCommand {
    fn run(&self, out: &mut dyn Write, matches: &ArgMatches, config: &Config) -> Result<()> {
        let count = matches.value_of("number").expect("required argument");
        let count = count
            .parse::<usize>()
            .map_err(|_| PastelError::CouldNotParseNumber(count.into()))?;
        if count < 2 {
            return Err(PastelError::ScaleNumberMustBeLargerThanOne);
        }

        let start = ColorArgIterator::from_color_arg(
            matches.value_of("color-start").expect("required argument"),
        )?;
        let stop = ColorArgIterator::from_color_arg(
            matches.value_of("color-stop").expect("required argument"),
        )?;

        for i in 0..count {
            let fraction = Fraction::from(i as f64 / (count as f64 - 1.0));

            let color = match matches.value_of("colorspace").expect("required argument") {
                "rgb" => start.mix::<RGBA<f64>>(&stop, fraction),
                "hsl" => start.mix::<HSLA>(&stop, fraction),
                "lab" => start.mix::<Lab>(&stop, fraction),
                "lch" => start.mix::<LCh>(&stop, fraction),
                _ => unimplemented!("Unknown color space"),
            };

            show_color(out, &config, &color)?;
        }

        Ok(())
    }
}
