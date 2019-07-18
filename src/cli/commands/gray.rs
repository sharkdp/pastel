use crate::commands::prelude::*;
use crate::commands::show::show_color;

use pastel::Color;

pub struct GrayCommand;

impl GenericCommand for GrayCommand {
    fn run(&self, out: &mut dyn Write, matches: &ArgMatches, config: &Config) -> Result<()> {
        let lightness = number_arg(matches, "lightness")?;
        let gray = Color::graytone(lightness);
        show_color(out, &config, &gray)
    }
}
