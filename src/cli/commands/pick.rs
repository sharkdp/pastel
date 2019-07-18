use crate::commands::prelude::*;
use crate::hdcanvas::Canvas;

pub struct PickCommand;

impl GenericCommand for PickCommand {
    fn run(&self, out: &mut dyn Write, _: &ArgMatches, config: &Config) -> Result<()> {
        let width = config.colorpicker_width;

        let mut canvas = Canvas::new(
            width + 2 * config.padding,
            width + 2 * config.padding,
            config.brush,
        );
        canvas.draw_rect(
            config.padding,
            config.padding,
            width + 2,
            width + 2,
            &Color::graytone(0.39),
        );

        for y in 0..width {
            for x in 0..width {
                let rx = (x as f64) / (width as f64);
                let ry = (y as f64) / (width as f64);

                let h = 360.0 * rx;
                let s = 0.6;
                let l = 0.81 * ry + 0.05;

                // Start with HSL
                let color = Color::from_hsl(h, s, l);

                // But (slightly) normalize the luminance
                let mut lch = color.to_lch();
                lch.l = (lch.l + ry * 100.0) / 2.0;
                let color = Color::from_lch(lch.l, lch.c, lch.h);

                canvas.draw_rect(config.padding + y + 1, config.padding + x + 1, 1, 1, &color);
            }
        }

        canvas.print(out)?;

        Ok(())
    }
}
