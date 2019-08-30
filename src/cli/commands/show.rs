use crate::commands::prelude::*;
use crate::output::Output;

pub struct ShowCommand;

impl ColorCommand for ShowCommand {
    fn run(
        &self,
        out: &mut dyn Write,
        _: &ArgMatches,
        config: &Config,
        color: &Color,
    ) -> Result<()> {
        (Output::new(out)).show_color(config, color)
    }
}
