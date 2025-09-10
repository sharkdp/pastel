use crate::commands::prelude::*;

use pastel::Color;

pub struct GrayCommand;

impl GenericCommand for GrayCommand {
    fn run(&self, out: &mut Output, matches: &ArgMatches, _config: &Config) -> Result<()> {
        let lightness = number_arg(matches, "lightness")?;
        let gray = Color::graytone(lightness);
        out.push_color(gray);
        Ok(())
    }
}
