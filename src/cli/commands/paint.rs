use crate::commands::prelude::*;
use crate::parser::parse_color;

pub struct PaintCommand;

impl GenericCommand for PaintCommand {
    fn run(&self, matches: &ArgMatches, _: &Config) -> Result<()> {
        let text = matches.value_of("text").expect("required argument");

        let fg = matches.value_of("color").expect("required argument");

        if fg.trim() != "default" {
            // TODO: remove duplication - move this into a function and use it in
            // color_args(). Write integration tests
            let fg = if fg == "-" {
                color_from_stdin()?
            } else {
                parse_color(fg).ok_or(PastelError::ColorParseError(fg.into()))?
            };

            let fg_rgba = fg.to_rgba();
            print!(
                "\x1b[38;2;{r};{g};{b}m",
                r = fg_rgba.r,
                g = fg_rgba.g,
                b = fg_rgba.b
            );
        }

        if let Some(bg) = matches.value_of("on") {
            let bg = parse_color(bg).ok_or(PastelError::ColorParseError(bg.into()))?;
            let bg_rgba = bg.to_rgba();
            print!(
                "\x1b[48;2;{r};{g};{b}m",
                r = bg_rgba.r,
                g = bg_rgba.g,
                b = bg_rgba.b
            );
        }

        if matches.is_present("bold") {
            print!("\x1b[1m")
        }

        if matches.is_present("italic") {
            print!("\x1b[3m")
        }

        if matches.is_present("underline") {
            print!("\x1b[4m")
        }

        print!("{}", text);
        print!("\x1b[0m");

        if !matches.is_present("no-newline") {
            println!();
        }

        Ok(())
    }
}
