use crate::commands::prelude::*;
use crate::utility::similar_colors;

use pastel::Format;

pub struct FormatCommand;

impl ColorCommand for FormatCommand {
    fn run(
        &self,
        out: &mut dyn Write,
        matches: &ArgMatches,
        config: &Config,
        color: &Color,
    ) -> Result<()> {
        let format_type = matches.value_of("type").expect("required argument");

        let output = match format_type {
            "rgb" => color.to_rgb_string(Format::Spaces),
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
            "name" => similar_colors(color)[0].name.to_owned(),
            &_ => {
                unreachable!("Unknown format type");
            }
        };

        writeln!(
            out,
            "{}",
            config
                .brush
                .paint(output, color.text_color().ansi_style().on(color))
        )?;

        Ok(())
    }
}
