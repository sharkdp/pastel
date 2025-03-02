use crate::commands::prelude::*;

pub struct SortCommand;

pub fn key_function(sort_order: &str, color: &Color) -> i32 {
    match sort_order {
        "brightness" => (color.brightness() * 1000.0) as i32,
        "luminance" => (color.luminance() * 1000.0) as i32,
        "hue" => (color.to_lch().h * 1000.0) as i32,
        "chroma" => (color.to_lch().c * 1000.0) as i32,
        "random" => rand::random(),
        _ => unreachable!("Unknown sort order"),
    }
}

impl GenericCommand for SortCommand {
    fn run(&self, out: &mut Output, matches: &ArgMatches, config: &Config) -> Result<()> {
        let sort_order = matches.value_of("sort-order").expect("required argument");

        let mut colors: Vec<Color> = vec![];
        for color in ColorArgIterator::from_args(config, matches.values_of("color"))? {
            colors.push(color?);
        }

        if matches.is_present("unique") {
            colors.sort_by_key(|c| c.to_u32());
            colors.dedup_by_key(|c| c.to_u32());
        }

        colors.sort_by_cached_key(|c| key_function(sort_order, c));

        if matches.is_present("reverse") {
            colors.reverse();
        }

        for color in colors {
            out.show_color(config, &color)?;
        }

        Ok(())
    }
}
