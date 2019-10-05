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

struct ColorPickerTool {
    command: &'static str,
    args: Vec<&'static str>,
    version_args: Vec<&'static str>,
    version_output_starts_with: &'static [u8],
}

/// Run an external color picker tool (e.g. gpick or xcolor) and get the output as a string.
pub fn run_external_colorpicker(picker: Option<&str>) -> Result<String> {
    let tools = [
        #[cfg(target_os = "macos")]
        ColorPickerTool {
            command: "osascript",
            // NOTE: This does not use `console.log` to print the value as you might expect,
            // because that gets written to stderr instead of stdout regardless of the `-s o` flag.
            // (This is accurate as of macOS Mojave/10.14.6).
            // See related: https://apple.stackexchange.com/a/278395
            args: vec![
                "-l",
                "JavaScript",
                "-s",
                "o",
                "-e",
                "
                const app = Application.currentApplication();\n
                app.includeStandardAdditions = true;\n
                const rgb = app.chooseColor({defaultColor: [0.5, 0.5, 0.5]})\n
                  .map(n => Math.round(n * 255))\n
                  .join(', ');\n
                `rgb(${rgb})`;\n
            ",
            ],
            version_args: vec!["-l", "JavaScript", "-s", "o", "-e", "'ok';"],
            version_output_starts_with: b"ok",
        },
        ColorPickerTool {
            command: "gpick",
            args: vec!["--pick", "--single", "--output"],
            version_args: vec!["--version"],
            version_output_starts_with: b"Gpick",
        },
        ColorPickerTool {
            command: "xcolor",
            args: vec!["--format", "hex"],
            version_args: vec!["--version"],
            version_output_starts_with: b"xcolor",
        },
        ColorPickerTool {
            command: "grabc",
            args: vec!["-hex"],
            version_args: vec!["-v"],
            version_output_starts_with: b"grabc",
        },
        ColorPickerTool {
            command: "colorpicker",
            args: vec!["--one-shot", "--short"],
            version_args: vec!["--help"],
            version_output_starts_with: b"",
        },
        ColorPickerTool {
            command: "chameleon",
            args: vec![],
            version_args: vec!["-h"],
            version_output_starts_with: b"",
        },
        ColorPickerTool {
            command: "kcolorchooser",
            args: vec!["--print"],
            version_args: vec!["-v"],
            version_output_starts_with: b"kcolorchooser",
        },
    ];

    for tool in tools
        .iter()
        .filter(|t| picker.map_or(true, |p| t.command.eq_ignore_ascii_case(p)))
    {
        let result = Command::new(tool.command).args(&tool.version_args).output();

        let tool_is_available = match result {
            Ok(ref output) => {
                output.stdout.starts_with(tool.version_output_starts_with)
                    || output.stderr.starts_with(tool.version_output_starts_with)
            }
            _ => false,
        };

        if tool_is_available {
            let result = Command::new(tool.command).args(&tool.args).output()?;
            if !result.status.success() {
                return Err(PastelError::ColorPickerExecutionError(
                    tool.command.to_string(),
                ));
            }

            let color =
                String::from_utf8(result.stdout).map_err(|_| PastelError::ColorInvalidUTF8)?;
            let color = color.trim();

            return Ok(color.to_string());
        }
    }

    return Err(PastelError::NoColorPickerFound);
}
