use crate::commands::prelude::*;

pub struct ContrastCommand;

impl GenericCommand for ContrastCommand {
    fn run(&self, out: &mut Output, matches: &ArgMatches, config: &Config) -> Result<()> {
        let colors: Vec<pastel::Color> =
            ColorArgIterator::from_args(config, matches.get_many::<String>("color"))?
                .collect::<Result<Vec<_>>>()?;

        if colors.len() < 2 {
            return Err(PastelError::ContrastColorCountMustBeLargerThanOne);
        }

        let base = &colors[0];

        if config.interactive_mode {
            writeln!(
                out.handle,
                "Base: {}",
                config.brush.paint(
                    format!("{}", &base.to_rgb_hex_string(true)),
                    base.text_color().ansi_style().on(base)
                )
            )?;
            writeln!(out.handle)?;
            for color in &colors[1..] {
                let ratio = base.contrast_ratio(color);
                let text = format!("{}  {:.2}:1", color.to_rgb_hex_string(true), ratio);
                writeln!(
                    out.handle,
                    "{}",
                    config.brush.paint(
                        format!("{}", &text),
                        color.text_color().ansi_style().on(color)
                    )
                )?;
            }
        } else {
            let ratios: Vec<String> = colors[1..]
                .iter()
                .map(|color| format!("{:.2}:1", base.contrast_ratio(color)))
                .collect();
            writeln!(out.handle, "{}", ratios.join("  "))?;
        }

        Ok(())
    }
}
