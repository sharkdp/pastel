use crate::commands::prelude::*;
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

        let replace_escape = |code: &str| code.replace('\x1b', "\\x1b");

        let output = match format_type.as_ref() {
            "rgb" => color.to_rgb_string(Format::Spaces),
            "rgb-float" => color.to_rgb_float_string(Format::Spaces),
            "hex" => color.to_rgb_hex_string(true),
            "hsl" => color.to_hsl_string(Format::Spaces),
            "hsl-hue" => format!("{:.0}", color.to_hsla().h),
            "hsl-saturation" => format!("{:.4}", color.to_hsla().s),
            "hsl-lightness" => format!("{:.4}", color.to_hsla().l),
            "hsv" => color.to_hsv_string(Format::Spaces),
            "hsv-hue" => format!("{:.0}", color.to_hsva().h),
            "hsv-saturation" => format!("{:.4}", color.to_hsva().s),
            "hsv-value" => format!("{:.4}", color.to_hsva().v),
            "lch" => color.to_lch_string(Format::Spaces),
            "lch-lightness" => format!("{:.2}", color.to_lch().l),
            "lch-chroma" => format!("{:.2}", color.to_lch().c),
            "lch-hue" => format!("{:.2}", color.to_lch().h),
            "lab" => color.to_lab_string(Format::Spaces),
            "lab-a" => format!("{:.2}", color.to_lab().a),
            "lab-b" => format!("{:.2}", color.to_lab().b),
            "oklab" => color.to_oklab_string(Format::Spaces),
            "oklab-l" => format!("{:.4}", color.to_oklab().l),
            "oklab-a" => format!("{:.4}", color.to_oklab().a),
            "oklab-b" => format!("{:.4}", color.to_oklab().b),
            "luminance" => format!("{:.3}", color.luminance()),
            "brightness" => format!("{:.3}", color.brightness()),
            "ansi-8bit" => replace_escape(&color.to_ansi_sequence(Mode::Ansi8Bit)),
            "ansi-24bit" => replace_escape(&color.to_ansi_sequence(Mode::TrueColor)),
            "ansi-8bit-escapecode" => color.to_ansi_sequence(Mode::Ansi8Bit),
            "ansi-24bit-escapecode" => color.to_ansi_sequence(Mode::TrueColor),
            "cmyk" => color.to_cmyk_string(Format::Spaces),
            "name" => similar_colors(color)[0].name.to_owned(),
            &_ => {
                unreachable!("Unknown format type");
            }
        };

        let write_colored_line = !matches!(
            format_type.as_ref(),
            "ansi-8bit-escapecode" | "ansi-24bit-escapecode"
        );

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
