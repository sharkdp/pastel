use std::io::{self, Write};

use crate::commands::prelude::*;
use crate::parser::parse_color;

use pastel::ansi::{Brush, Mode, Style};

pub struct PaintCommand;

impl GenericCommand for PaintCommand {
    fn run(&self, matches: &ArgMatches, _: &Config) -> Result<()> {
        let text = matches.value_of("text").expect("required argument");

        let fg = matches.value_of("color").expect("required argument");
        let fg = if fg.trim() == "default" {
            None
        } else {
            // TODO: remove duplication - move this into a function and use it in
            // color_args(). Write integration tests
            if fg == "-" {
                Some(color_from_stdin()?)
            } else {
                Some(parse_color(fg).ok_or(PastelError::ColorParseError(fg.into()))?)
            }
        };

        let bg = if let Some(bg) = matches.value_of("on") {
            Some(parse_color(bg).ok_or(PastelError::ColorParseError(bg.into()))?)
        } else {
            None
        };

        let mut style = Style::default();

        if let Some(fg) = fg {
            style.foreground(&fg);
        }

        if let Some(bg) = bg {
            style.on(&bg);
        }

        style.bold(matches.is_present("bold"));
        style.italic(matches.is_present("italic"));
        style.underline(matches.is_present("underline"));

        let stdout = io::stdout();

        writeln!(
            stdout.lock(),
            "{}{}",
            Brush::from_mode(Mode::TrueColor).paint(text, &style),
            if matches.is_present("no-newline") {
                ""
            } else {
                "\n"
            }
        )
        .map_err(|_| PastelError::StdoutClosed)?;

        Ok(())
    }
}
