use crate::commands::prelude::*;
use crate::commands::sort::key_function;
use crate::named::{NamedColor, NAMED_COLORS};

use pastel::ansi::ToAnsiStyle;

pub struct ListCommand;

impl GenericCommand for ListCommand {
    fn run(&self, out: &mut Output, matches: &ArgMatches, config: &Config) -> Result<()> {
        let sort_order = matches.value_of("sort-order").expect("required argument");

        let mut colors: Vec<&NamedColor> = NAMED_COLORS.iter().map(|r| r).collect();
        colors.sort_by_key(|nc| key_function(sort_order, &nc.color));
        colors.dedup_by(|n1, n2| n1.color == n2.color);

        if config.interactive_mode {
            for nc in colors {
                let bg = &nc.color;
                let fg = bg.text_color();
                writeln!(
                    out.handle,
                    "{}",
                    config
                        .brush
                        .paint(format!(" {:24}", nc.name), fg.ansi_style().on(bg))
                )?;
            }
        } else {
            for nc in colors {
                let res = writeln!(out.handle, "{}", nc.name);
                if res.is_err() {
                    break;
                }
            }
        }

        Ok(())
    }
}
