use crate::config::Config;
use crate::output::Output;
use crate::Result;

use clap::ArgMatches;

use pastel::Color;

pub trait GenericCommand {
    fn run(&self, out: &mut Output, matches: &ArgMatches, config: &Config) -> Result<()>;
}

pub trait ColorCommand {
    fn run(
        &self,
        out: &mut Output,
        matches: &ArgMatches,
        config: &Config,
        color: &Color,
    ) -> Result<()>;
}
