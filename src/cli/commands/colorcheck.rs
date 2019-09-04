use crate::commands::prelude::*;
use crate::hdcanvas::Canvas;

use pastel::ansi::{Brush, Mode};

pub struct ColorCheckCommand;

fn print_board(out: &mut Output, config: &Config, mode: Mode) -> Result<()> {
    // These colors have been chosen/computed such that the perceived color difference (CIE delta-E
    // 2000) to the closest ANSI 8-bit color is maximal.
    let c1 = Color::from_rgb(73, 39, 50);
    let c2 = Color::from_rgb(16, 51, 30);
    let c3 = Color::from_rgb(29, 54, 90);

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

    canvas.print(out.handle)
}

impl GenericCommand for ColorCheckCommand {
    fn run(&self, out: &mut Output, _: &ArgMatches, config: &Config) -> Result<()> {
        writeln!(out.handle, "\n8-bit mode:")?;
        print_board(out, config, Mode::Ansi8Bit)?;

        writeln!(out.handle, "24-bit mode:")?;
        print_board(out, config, Mode::TrueColor)?;

        writeln!(
            out.handle,
            "If your terminal emulator supports 24-bit colors, you should see three square color \
             panels in the lower row and the colors should look similar (but slightly different \
             from) the colors in the top row panels.\nThe panels in the lower row should look \
             like squares that are filled with a uniform color (no stripes or other artifacts).\n\
             \n\
             You can also open https://github.com/sharkdp/pastel/blob/master/doc/colorcheck.md in \
             a browser to compare how the output should look like."
        )?;

        Ok(())
    }
}
