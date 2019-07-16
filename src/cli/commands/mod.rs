use crate::config::Config;
use crate::error::Result;
use clap::ArgMatches;

mod color_commands;
mod format;
mod gray;
mod io;
mod list;
mod paint;
mod pick;
mod prelude;
mod random;
mod show;
mod traits;

use traits::{ColorCommand, GenericCommand};

use format::FormatCommand;
use gray::GrayCommand;
use list::ListCommand;
use paint::PaintCommand;
use pick::PickCommand;
use random::RandomCommand;

use io::color_args;

pub enum Command {
    WithColor(Box<dyn ColorCommand>),
    Generic(Box<dyn GenericCommand>),
}

impl Command {
    pub fn from_string(command: &str) -> Command {
        match command {
            "show" => Command::WithColor(Box::new(show::ShowCommand)),
            "saturate" => Command::WithColor(Box::new(color_commands::SaturateCommand)),
            "desaturate" => Command::WithColor(Box::new(color_commands::DesaturateCommand)),
            "lighten" => Command::WithColor(Box::new(color_commands::LightenCommand)),
            "darken" => Command::WithColor(Box::new(color_commands::DarkenCommand)),
            "rotate" => Command::WithColor(Box::new(color_commands::RotateCommand)),
            "complement" => Command::WithColor(Box::new(color_commands::ComplementCommand)),
            "to-gray" => Command::WithColor(Box::new(color_commands::ToGrayCommand)),
            "pick" => Command::Generic(Box::new(PickCommand)),
            "gray" => Command::Generic(Box::new(GrayCommand)),
            "list" => Command::Generic(Box::new(ListCommand)),
            "random" => Command::Generic(Box::new(RandomCommand)),
            "paint" => Command::Generic(Box::new(PaintCommand)),
            "format" => Command::WithColor(Box::new(FormatCommand)),
            _ => unreachable!("Unknown subcommand"),
        }
    }

    pub fn execute(&self, matches: &ArgMatches, config: &Config) -> Result<()> {
        match self {
            Command::Generic(cmd) => cmd.run(matches, config),
            Command::WithColor(cmd) => {
                for color in color_args(matches)? {
                    cmd.run(matches, config, &color)?;
                }

                Ok(())
            }
        }
    }
}
