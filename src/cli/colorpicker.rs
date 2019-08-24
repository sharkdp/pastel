use std::io::{self, Write};
use std::process::Command;

use crate::config::Config;
use crate::error::{PastelError, Result};
use crate::hdcanvas::Canvas;

use pastel::ansi::{Brush, Stream};
use pastel::Color;

/// Print a color spectrum to STDERR.
pub fn print_colorspectrum(config: &Config) -> Result<()> {
    let width = config.colorpicker_width;

    let mut canvas = Canvas::new(
        width + 2 * config.padding,
        width + 2 * config.padding,
        Brush::from_environment(Stream::Stderr),
    );
    canvas.draw_rect(
        config.padding,
        config.padding,
        width + 2,
        width + 2,
        &Color::white(),
    );

    for y in 0..width {
        for x in 0..width {
            let rx = (x as f64) / (width as f64);
            let ry = (y as f64) / (width as f64);

            let h = 360.0 * rx;
            let s = 0.6;
            let l = 0.95 * ry;

            // Start with HSL
            let color = Color::from_hsl(h, s, l);

            // But (slightly) normalize the luminance
            let mut lch = color.to_lch();
            lch.l = (lch.l + ry * 100.0) / 2.0;
            let color = Color::from_lch(lch.l, lch.c, lch.h, 1.0);

            canvas.draw_rect(config.padding + y + 1, config.padding + x + 1, 1, 1, &color);
        }
    }

    let stderr_handle = io::stderr();
    let mut stderr = stderr_handle.lock();

    canvas.print(&mut stderr)?;
    writeln!(&mut stderr)?;
    Ok(())
}

/// Run an external X11 color picker tool (e.g. xcolor) and get the output as a string.
///
/// TODO: support more tools
pub fn run_external_colorpicker() -> Result<String> {
    let result = Command::new("xcolor").arg("--version").output();
    match result {
        Ok(ref output) if output.status.success() && output.stdout.starts_with(b"xcolor") => {}
        _ => return Err(PastelError::NoColorPickerFound),
    }

    let result = Command::new("xcolor").arg("--format").arg("hex").output()?;
    if !result.status.success() {
        return Err(PastelError::NoColorPickerFound);
    }

    let color = String::from_utf8(result.stdout).map_err(|_| PastelError::ColorInvalidUTF8)?;
    let color = color.trim();

    Ok(color.to_string())
}
