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
        let format_type = matches
            .get_one::<String>("type")
            .expect("required argument");
        let format_type = format_type.to_lowercase();

        let braced: bool = *matches.get_one("braced").unwrap_or(&false);

        let output_format = if braced {
            Format::Braced
        } else {
            Format::Spaces
        };

        let replace_escape = |code: &str| code.replace('\x1b', "\\x1b");

        let output = match format_type.as_ref() {
            "rgb" => color.to_rgb_string(output_format),
            "rgb-float" => color.to_rgb_float_string(output_format),
            "rgb-r" => format!("{}", color.to_rgba().r),
            "rgb-g" => format!("{}", color.to_rgba().g),
            "rgb-b" => format!("{}", color.to_rgba().b),
            "hex" => color.to_rgb_hex_string(true),
            "hsl" => color.to_hsl_string(output_format),
            "hsl-hue" => format!("{:.0}", color.to_hsla().h),
            "hsl-saturation" => format!("{:.4}", color.to_hsla().s),
            "hsl-lightness" => format!("{:.4}", color.to_hsla().l),
            "hsv" => color.to_hsv_string(output_format),
            "hsv-hue" => format!("{:.0}", color.to_hsva().h),
            "hsv-saturation" => format!("{:.4}", color.to_hsva().s),
            "hsv-value" => format!("{:.4}", color.to_hsva().v),
            "lch" => color.to_lch_string(output_format),
            "lch-lightness" => format!("{:.2}", color.to_lch().l),
            "lch-chroma" => format!("{:.2}", color.to_lch().c),
            "lch-hue" => format!("{:.2}", color.to_lch().h),
            "lab" => color.to_lab_string(output_format),
            "lab-a" => format!("{:.2}", color.to_lab().a),
            "lab-b" => format!("{:.2}", color.to_lab().b),
            "oklch" => color.to_oklch_string(output_format),
            "oklch-lightness" => format!("{:.4}", color.to_oklch().l),
            "oklch-chroma" => format!("{:.4}", color.to_oklch().c),
            "oklch-hue" => format!("{:.4}", color.to_oklch().h),
            "oklab" => color.to_oklab_string(output_format),
            "oklab-l" => format!("{:.4}", color.to_oklab().l),
            "oklab-a" => format!("{:.4}", color.to_oklab().a),
            "oklab-b" => format!("{:.4}", color.to_oklab().b),
            "luminance" => format!("{:.3}", color.luminance()),
            "brightness" => format!("{:.3}", color.brightness()),
            "ansi-8bit" => replace_escape(&color.to_ansi_sequence(Mode::Ansi8Bit)),
            "ansi-24bit" => replace_escape(&color.to_ansi_sequence(Mode::TrueColor)),
            "ansi-8bit-value" => color.to_ansi_8bit().to_string() + "\n",
            "ansi-8bit-escapecode" => color.to_ansi_sequence(Mode::Ansi8Bit),
            "ansi-24bit-escapecode" => color.to_ansi_sequence(Mode::TrueColor),
            "cmyk" => color.to_cmyk_string(output_format),
            "name" => similar_colors(color)[0].name.to_owned(),
            &_ => {
                unreachable!("Unknown format type");
            }
        };

        let write_colored_line = !matches!(
            format_type.as_ref(),
            "ansi-8bit-escapecode" | "ansi-24bit-escapecode" | "ansi-8bit-value"
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
