use crate::commands::prelude::*;

use crate::colorpicker::{print_colorspectrum, run_external_colorpicker};

pub struct PickCommand;

impl GenericCommand for PickCommand {
    fn run(&self, out: &mut Output, matches: &ArgMatches, config: &Config) -> Result<()> {
        let count = matches.value_of("count").expect("required argument");
        let count = count
            .parse::<usize>()
            .map_err(|_| PastelError::CouldNotParseNumber(count.into()))?;

        print_colorspectrum(config)?;

        let mut color_strings = Vec::new();
        for _ in 0..count {
            color_strings.push(run_external_colorpicker(config.colorpicker)?);
        }

        let mut print_spectrum = PrintSpectrum::No;

        for color_str in color_strings {
            let color = ColorArgIterator::from_color_arg(config, &color_str, &mut print_spectrum)?;
            out.show_color(config, &color)?;
        }

        Ok(())
    }
}
