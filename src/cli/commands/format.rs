use crate::commands::prelude::*;

use pastel::ansi::{AnsiColor, Mode};
use pastel::Format;

pub struct FormatCommand;

impl ColorCommand for FormatCommand {
    fn run(&self, matches: &ArgMatches, _: &Config, color: &Color) -> Result<()> {
        let format_type = matches.value_of("type").expect("required argument");

        match format_type {
            "rgb" => {
                println!("{}", color.to_rgb_string(Format::Spaces));
            }
            "hex" => {
                println!("{}", color.to_rgb_hex_string());
            }
            "hsl" => {
                println!("{}", color.to_hsl_string(Format::Spaces));
            }
            "Lab" => {
                println!("{}", color.to_lab_string(Format::Spaces));
            }
            "LCh" => {
                println!("{}", color.to_lch_string(Format::Spaces));
            }
            "ansi-8-bit" => {
                print!("{}", color.to_ansi_sequence(Mode::Ansi8Bit));
            }
            "ansi-24-bit" => {
                print!("{}", color.to_ansi_sequence(Mode::TrueColor));
            }
            &_ => {
                unreachable!("Unknown format type");
            }
        }

        Ok(())
    }
}
