use std::io::{self, Write};
use std::process::Command;

use crate::colorpicker_tools::COLOR_PICKER_TOOLS;
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
        Brush::from_environment(Stream::Stderr)?,
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

/// Run an external color picker tool (e.g. gpick or xcolor) and get the output as a string.
pub fn run_external_colorpicker(picker: Option<&str>) -> Result<String> {
    for tool in COLOR_PICKER_TOOLS
        .iter()
        .filter(|t| picker.is_none_or(|p| t.command.eq_ignore_ascii_case(p)))
    {
        let result = Command::new(tool.command).args(tool.version_args).output();

        let tool_is_available = match result {
            Ok(ref output) => {
                output.stdout.starts_with(tool.version_output_starts_with)
                    || output.stderr.starts_with(tool.version_output_starts_with)
            }
            _ => false,
        };

        if tool_is_available {
            let result = Command::new(tool.command).args(tool.args).output()?;
            if !result.status.success() {
                return Err(PastelError::ColorPickerExecutionError(
                    tool.command.to_string(),
                ));
            }

            let color =
                String::from_utf8(result.stdout).map_err(|_| PastelError::ColorInvalidUTF8)?;
            let color = color.trim().to_string();

            // Check if tool requires some post processing of the output
            if let Some(post_process) = tool.post_process {
                return post_process(color)
                    .map_err(|error| PastelError::ColorParseError(error.to_string()));
            } else {
                return Ok(color);
            }
        }
    }

    Err(PastelError::NoColorPickerFound)
}
