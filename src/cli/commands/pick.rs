use std::io::Write;

use crate::commands::prelude::*;
use crate::output::Output;

use crate::colorpicker::{print_colorspectrum, run_external_colorpicker};

pub struct PickCommand;

impl GenericCommand for PickCommand {
    fn run(&self, out: &mut dyn Write, _: &ArgMatches, config: &Config) -> Result<()> {
        let mut o = Output::new(out);
        print_colorspectrum(config)?;
        let color_str = run_external_colorpicker()?;

        let mut print_spectrum = PrintSpectrum::No;
        let color = ColorArgIterator::from_color_arg(config, &color_str, &mut print_spectrum)?;
        o.show_color(config, &color)?;

        Ok(())
    }
}
