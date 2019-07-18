use std::io::{self, Write};

use crate::commands::prelude::*;
use crate::named::{NamedColor, NAMED_COLORS};

use pastel::ansi::ToAnsiStyle;

pub struct ListCommand;

impl GenericCommand for ListCommand {
    fn run(&self, out: &mut dyn Write, matches: &ArgMatches, config: &Config) -> Result<()> {
        let sort_order = matches.value_of("sort").expect("required argument");

        let mut colors: Vec<&NamedColor> = NAMED_COLORS.iter().map(|r| r).collect();
        if sort_order == "brightness" {
            colors.sort_by_key(|nc| (-nc.color.brightness() * 1000.0) as i32);
        } else if sort_order == "luminance" {
            colors.sort_by_key(|nc| (-nc.color.luminance() * 1000.0) as i32);
        } else if sort_order == "hue" {
            colors.sort_by_key(|nc| (nc.color.to_lch().h * 1000.0) as i32);
        } else if sort_order == "chroma" {
            colors.sort_by_key(|nc| (nc.color.to_lch().c * 1000.0) as i32);
        }
        colors.dedup_by(|n1, n2| n1.color == n2.color);

        if config.interactive_mode {
            for nc in colors {
                let bg = &nc.color;
                let fg = bg.text_color();
                writeln!(
                    out,
                    "{}",
                    config
                        .brush
                        .paint(format!(" {:24}", nc.name), fg.ansi_style().on(bg))
                )?;
            }
        } else {
            let stdout = io::stdout();
            let mut out = stdout.lock();
            for nc in colors {
                let res = writeln!(out, "{}", nc.name);
                if res.is_err() {
                    break;
                }
            }
        }

        Ok(())
    }
}
