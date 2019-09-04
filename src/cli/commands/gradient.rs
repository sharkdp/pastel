use crate::colorspace::get_mixing_function;
use crate::commands::prelude::*;

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

        let start = ColorArgIterator::from_color_arg(
            config,
            matches.value_of("color-start").expect("required argument"),
            &mut print_spectrum,
        )?;
        let stop = ColorArgIterator::from_color_arg(
            config,
            matches.value_of("color-stop").expect("required argument"),
            &mut print_spectrum,
        )?;

        let mix = get_mixing_function(matches.value_of("colorspace").expect("required argument"));

        for i in 0..count {
            let fraction = Fraction::from(i as f64 / (count as f64 - 1.0));
            let color = mix(&start, &stop, fraction);

            out.show_color(&config, &color)?;
        }

        Ok(())
    }
}
