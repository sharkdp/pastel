use std::io::{self, Read};

use crate::commands::prelude::*;

use super::io::ColorArgIterator;

use pastel::ansi::Style;
use pastel::parser::parse_color;

pub struct PaintCommand;

impl GenericCommand for PaintCommand {
    fn run(&self, out: &mut Output, matches: &ArgMatches, config: &Config) -> Result<()> {
        let fg = matches
            .get_one::<String>("color")
            .expect("required argument");
        let fg = if fg.trim() == "default" {
            None
        } else {
            let mut print_spectrum = PrintSpectrum::Yes;
            Some(ColorArgIterator::from_color_arg(
                config,
                fg,
                &mut print_spectrum,
            )?)
        };

        let bg = if let Some(bg) = matches.get_one::<String>("on") {
            Some(parse_color(bg).ok_or_else(|| PastelError::ColorParseError(bg.into()))?)
        } else {
            None
        };

        let text = match matches.get_many::<String>("text") {
            Some(values) => values.cloned().collect::<Vec<_>>().join(" "),
            _ => {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                buffer
            }
        };

        let mut style = Style::default();

        if let Some(fg) = fg {
            style.foreground(&fg);
        }

        if let Some(bg) = bg {
            style.on(bg);
        }

        style.bold(matches.get_flag("bold"));
        style.italic(matches.get_flag("italic"));
        style.underline(matches.get_flag("underline"));

        write!(
            out.handle,
            "{}{}",
            config.brush.paint(text, style),
            if matches.get_flag("no-newline") {
                ""
            } else {
                "\n"
            }
        )?;

        Ok(())
    }
}
