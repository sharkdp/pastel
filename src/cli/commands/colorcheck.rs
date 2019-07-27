use crate::commands::prelude::*;
use crate::hdcanvas::Canvas;

use pastel::ansi::{Brush, Mode};

pub struct ColorCheckCommand;

fn print_board(out: &mut dyn Write, config: &Config, mode: Mode) -> Result<()> {
    let c1 = Color::from_rgb(127, 68, 32);
    let c2 = Color::from_rgb(83, 127, 48);
    let c3 = Color::from_rgb(7, 47, 127);

    let width = config.colorcheck_width;

    let mut canvas = Canvas::new(
        width + 2 * config.padding,
        3 * width + 3 * config.padding,
        Brush::from_mode(Some(mode)),
    );

    canvas.draw_rect(config.padding, config.padding, width, width, &c1);

    canvas.draw_rect(
        config.padding,
        2 * config.padding + width,
        width,
        width,
        &c2,
    );

    canvas.draw_rect(
        config.padding,
        3 * config.padding + 2 * width,
        width,
        width,
        &c3,
    );

    canvas.print(out)
}

impl GenericCommand for ColorCheckCommand {
    fn run(&self, out: &mut dyn Write, _: &ArgMatches, config: &Config) -> Result<()> {
        writeln!(out, "\n8-bit mode:")?;
        print_board(out, config, Mode::Ansi8Bit)?;

        writeln!(out, "24-bit mode:")?;
        print_board(out, config, Mode::TrueColor)?;

        writeln!(
            out,
            "If your terminal emulator supports 24-bit colors, you should see three square color \
             panels in the lower row and the colors should look similar (but different from) \
             the colors in the top row panels."
        )?;

        Ok(())
    }
}
