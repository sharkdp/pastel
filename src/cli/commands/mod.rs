use crate::config::Config;
use crate::error::Result;
use crate::output::Output;
use clap::ArgMatches;

mod color_commands;
mod colorcheck;
mod distinct;
mod format;
mod gradient;
mod gray;
mod io;
mod list;
mod paint;
mod pick;
mod prelude;
mod random;
mod show;
mod sort;
mod traits;

use traits::{ColorCommand, GenericCommand};

use colorcheck::ColorCheckCommand;
use distinct::DistinctCommand;
use format::FormatCommand;
use gradient::GradientCommand;
use gray::GrayCommand;
use list::ListCommand;
use paint::PaintCommand;
use pick::PickCommand;
use random::RandomCommand;
use sort::SortCommand;

use io::ColorArgIterator;

pub enum Command {
    WithColor(Box<dyn ColorCommand>),
    Generic(Box<dyn GenericCommand>),
}

impl Command {
    pub fn from_string(command: &str) -> Command {
        match command {
            "color" => Command::WithColor(Box::new(show::ShowCommand)),
            "saturate" => Command::WithColor(Box::new(color_commands::SaturateCommand)),
            "desaturate" => Command::WithColor(Box::new(color_commands::DesaturateCommand)),
            "lighten" => Command::WithColor(Box::new(color_commands::LightenCommand)),
            "darken" => Command::WithColor(Box::new(color_commands::DarkenCommand)),
            "rotate" => Command::WithColor(Box::new(color_commands::RotateCommand)),
            "colorblind" => Command::WithColor(Box::new(color_commands::ColorblindCommand)),
            "set" => Command::WithColor(Box::new(color_commands::SetCommand)),
            "complement" => Command::WithColor(Box::new(color_commands::ComplementCommand)),
            "mix" => Command::WithColor(Box::new(color_commands::MixCommand)),
            "to-gray" => Command::WithColor(Box::new(color_commands::ToGrayCommand)),
            "textcolor" => Command::WithColor(Box::new(color_commands::TextColorCommand)),
            "pick" => Command::Generic(Box::new(PickCommand)),
            "gray" => Command::Generic(Box::new(GrayCommand)),
            "list" => Command::Generic(Box::new(ListCommand)),
            "sort-by" => Command::Generic(Box::new(SortCommand)),
            "random" => Command::Generic(Box::new(RandomCommand)),
            "distinct" => Command::Generic(Box::new(DistinctCommand)),
            "gradient" => Command::Generic(Box::new(GradientCommand)),
            "paint" => Command::Generic(Box::new(PaintCommand)),
            "format" => Command::WithColor(Box::new(FormatCommand)),
            "colorcheck" => Command::Generic(Box::new(ColorCheckCommand)),
            _ => unreachable!("Unknown subcommand"),
        }
    }

    pub fn execute(&self, matches: &ArgMatches, config: &Config) -> Result<()> {
        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();
        let mut out = Output::new(&mut stdout_lock);

        match self {
            Command::Generic(cmd) => cmd.run(&mut out, matches, config),
            Command::WithColor(cmd) => {
                for color in ColorArgIterator::from_args(config, matches.values_of("color"))? {
                    cmd.run(&mut out, matches, config, &color?)?;
                }

                Ok(())
            }
        }
    }
}
