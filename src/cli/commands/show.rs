use crate::commands::prelude::*;
use crate::output::Output;

pub struct ShowCommand;

impl ColorCommand for ShowCommand {
    fn run(
        &self,
        out: &mut Output,
        _: &ArgMatches,
        config: &Config,
        color: &Color,
    ) -> Result<()> {
        out.show_color(config, color)
    }
}
