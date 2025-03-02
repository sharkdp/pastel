use std::io::{self, BufRead};

use clap::{ArgMatches, Values};

use crate::colorpicker::{print_colorspectrum, run_external_colorpicker};
use crate::config::Config;
use crate::{PastelError, Result};

use pastel::parser::parse_color;
use pastel::Color;

pub fn number_arg(matches: &ArgMatches, name: &str) -> Result<f64> {
    let value_str = matches.value_of(name).expect("required argument");
    value_str
        .parse::<f64>()
        .map_err(|_| PastelError::CouldNotParseNumber(value_str.into()))
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrintSpectrum {
    Yes,
    No,
}

pub enum ColorArgIterator<'a> {
    FromPositionalArguments(&'a Config<'a>, Values<'a>, PrintSpectrum),
    FromStdin,
}

impl<'a> ColorArgIterator<'a> {
    pub fn from_args(config: &'a Config, args: Option<Values<'a>>) -> Result<Self> {
        match args {
            Some(positionals) => Ok(ColorArgIterator::FromPositionalArguments(
                config,
                positionals,
                PrintSpectrum::Yes,
            )),
            None => {
                use atty::Stream;
                if atty::is(Stream::Stdin) {
                    return Err(PastelError::ColorArgRequired);
                }
                Ok(ColorArgIterator::FromStdin)
            }
        }
    }

    pub fn color_from_stdin() -> Result<Color> {
        let stdin = io::stdin();
        let mut lock = stdin.lock();

        let mut line = String::new();
        let size = lock
            .read_line(&mut line)
            .map_err(|_| PastelError::ColorInvalidUTF8)?;

        if size == 0 {
            return Err(PastelError::CouldNotReadFromStdin);
        }

        let line = line.trim();

        parse_color(line).ok_or_else(|| PastelError::ColorParseError(line.to_string()))
    }

    pub fn from_color_arg(
        config: &'a Config,
        arg: &str,
        print_spectrum: &mut PrintSpectrum,
    ) -> Result<Color> {
        match arg {
            "-" => Self::color_from_stdin(),
            "pick" => {
                if *print_spectrum == PrintSpectrum::Yes {
                    print_colorspectrum(config)?;
                    *print_spectrum = PrintSpectrum::No;
                }
                let color_str = run_external_colorpicker(config.colorpicker)?;
                ColorArgIterator::from_color_arg(config, &color_str, print_spectrum)
            }
            color_str => {
                parse_color(color_str).ok_or_else(|| PastelError::ColorParseError(color_str.into()))
            }
        }
    }
}

impl Iterator for ColorArgIterator<'_> {
    type Item = Result<Color>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ColorArgIterator::FromPositionalArguments(
                ref mut config,
                ref mut args,
                ref mut print_spectrum,
            ) => args
                .next()
                .map(|color_arg| Self::from_color_arg(config, color_arg, print_spectrum)),

            ColorArgIterator::FromStdin => match Self::color_from_stdin() {
                Ok(color) => Some(Ok(color)),
                Err(PastelError::CouldNotReadFromStdin) => None,
                err @ Err(_) => Some(err),
            },
        }
    }
}
