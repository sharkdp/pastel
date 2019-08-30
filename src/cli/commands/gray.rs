use std::io::Write;

use crate::commands::prelude::*;
use crate::output::Output;

use pastel::Color;

pub struct GrayCommand;

impl GenericCommand for GrayCommand {
    fn run(&self, out: &mut dyn Write, matches: &ArgMatches, config: &Config) -> Result<()> {
        let mut o = Output::new(out);
        let lightness = number_arg(matches, "lightness")?;
        let gray = Color::graytone(lightness);
        o.show_color(&config, &gray)
    }
}
