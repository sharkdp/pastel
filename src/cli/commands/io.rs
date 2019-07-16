use std::io::{self, BufRead};

use atty::Stream;
use clap::ArgMatches;

use crate::parser::parse_color;
use crate::{PastelError, Result};

use pastel::Color;

pub fn number_arg(matches: &ArgMatches, name: &str) -> Result<f64> {
    let value_str = matches.value_of(name).unwrap();
    value_str
        .parse::<f64>()
        .map_err(|_| PastelError::CouldNotParseNumber(value_str.into()))
}

pub fn color_from_stdin() -> Result<Color> {
    // TODO: remove duplication between color_from_stdin and colors_from_stdin
    let stdin = io::stdin();
    let mut lock = stdin.lock();

    let mut line = String::new();
    let size = lock
        .read_line(&mut line)
        .map_err(|_| PastelError::ColorInvalidUTF8)?;

    if size == 0 {
        return Err(PastelError::CouldNotReadFromStdin);
    }

    parse_color(&line).ok_or(PastelError::ColorParseError(line.clone()))
}

pub fn colors_from_stdin() -> Result<Vec<Color>> {
    let stdin = io::stdin();
    let lock = stdin.lock();

    let colors = lock
        .lines()
        .map(|line| {
            let line = line.map_err(|_| PastelError::ColorInvalidUTF8)?;
            parse_color(&line).ok_or(PastelError::ColorParseError(line.clone()))
        })
        .collect::<Result<Vec<_>>>()?;

    if colors.is_empty() {
        return Err(PastelError::CouldNotReadFromStdin);
    }

    Ok(colors)
}

pub fn color_args(matches: &ArgMatches) -> Result<Vec<Color>> {
    if let Some(color_args) = matches.values_of("color") {
        color_args
            .map(|c| {
                if c == "-" {
                    color_from_stdin()
                } else {
                    parse_color(c).ok_or(PastelError::ColorParseError(c.into()))
                }
            })
            .collect()
    } else {
        if atty::is(Stream::Stdin) {
            return Err(PastelError::ColorArgRequired);
        }

        colors_from_stdin()
    }
}
