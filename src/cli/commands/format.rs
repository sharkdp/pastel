use crate::commands::prelude::*;
use crate::output::Output;
use crate::utility::similar_colors;

use pastel::ansi::Mode;
use pastel::Format;

pub struct FormatCommand;

impl ColorCommand for FormatCommand {
    fn run(
        &self,
        out: &mut Output,
        matches: &ArgMatches,
        config: &Config,
        color: &Color,
    ) -> Result<()> {
        let format_type = matches.value_of("type").expect("required argument");
        let format_type = format_type.to_lowercase();

        let output = match format_type.as_ref() {
            "rgb" => color.to_rgb_string(Format::Spaces),
            "rgb-float" => color.to_rgb_float_string(Format::Spaces),
            "hex" => color.to_rgb_hex_string(true),
            "hsl" => color.to_hsl_string(Format::Spaces),
            "hsl-hue" => format!("{:.0}", color.to_hsla().h),
            "hsl-saturation" => format!("{:.4}", color.to_hsla().s),
            "hsl-lightness" => format!("{:.4}", color.to_hsla().l),
            "lch" => color.to_lch_string(Format::Spaces),
            "lch-lightness" => format!("{:.2}", color.to_lch().l),
            "lch-chroma" => format!("{:.2}", color.to_lch().c),
            "lch-hue" => format!("{:.2}", color.to_lch().h),
            "lab" => color.to_lab_string(Format::Spaces),
            "lab-a" => format!("{:.2}", color.to_lab().a),
            "lab-b" => format!("{:.2}", color.to_lab().b),
            "luminance" => format!("{:.3}", color.luminance()),
            "brightness" => format!("{:.3}", color.brightness()),
            "ansi-8bit" => color.to_ansi_sequence(Mode::Ansi8Bit),
            "ansi-24bit" => color.to_ansi_sequence(Mode::TrueColor),
            "name" => similar_colors(color)[0].name.to_owned(),
            &_ => {
                unreachable!("Unknown format type");
            }
        };

        let write_colored_line = match format_type.as_ref() {
            "ansi-8bit" | "ansi-24bit" => false,
            _ => true,
        };

        if write_colored_line {
            writeln!(
                out.handle,
                "{}",
                config
                    .brush
                    .paint(output, color.text_color().ansi_style().on(color))
            )?;
        } else {
            write!(out.handle, "{}", output)?;
        }

        Ok(())
    }
}
