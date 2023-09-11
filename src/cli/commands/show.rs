use crate::commands::prelude::*;

pub struct ShowCommand;

impl ColorCommand for ShowCommand {
    fn run(&self, out: &mut Output, _: &ArgMatches, _config: &Config, color: &Color) -> Result<()> {
        out.push_color(color.clone());
        Ok(())
    }
}
