use crate::commands::prelude::*;

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
            "hex" => color.to_rgb_hex_string(),
            "hsl" => color.to_hsl_string(Format::Spaces),
            "lab" => color.to_lab_string(Format::Spaces),
            "lch" => color.to_lch_string(Format::Spaces),
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
