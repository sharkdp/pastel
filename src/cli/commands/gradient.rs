use crate::colorspace::get_mixing_function;
use crate::commands::prelude::*;

use pastel::ColorScale;
use pastel::Fraction;

pub struct GradientCommand;

impl GenericCommand for GradientCommand {
    fn run(&self, out: &mut Output, matches: &ArgMatches, config: &Config) -> Result<()> {
        let count = matches.value_of("number").expect("required argument");
        let count = count
            .parse::<usize>()
            .map_err(|_| PastelError::CouldNotParseNumber(count.into()))?;
        if count < 2 {
            return Err(PastelError::GradientNumberMustBeLargerThanOne);
        }

        let mut print_spectrum = PrintSpectrum::Yes;

        let mix = get_mixing_function(matches.value_of("colorspace").expect("required argument"));

        let colors = matches
            .values_of("color")
            .expect("required argument")
            .map(|color| ColorArgIterator::from_color_arg(config, color, &mut print_spectrum));

        let color_count = colors.len();
        if color_count < 2 {
            return Err(PastelError::GradientColorCountMustBeLargerThanOne);
        }

        let mut color_scale = ColorScale::empty();

        for (i, color) in colors.enumerate() {
            let position = Fraction::from(i as f64 / (color_count as f64 - 1.0));

            color_scale.add_stop(color?, position);
        }

        for i in 0..count {
            let position = Fraction::from(i as f64 / (count as f64 - 1.0));

            let color = color_scale.sample(position, &mix).expect("gradient color");

            out.show_color(config, &color)?;
        }

        Ok(())
    }
}
