use crate::commands::prelude::*;

pub struct SortBinaryOperationCommand;

pub fn key_function(binary_operation: &str, operand: &Color, color: &Color) -> i32 {
    match binary_operation {
        "contrast" => (operand.contrast_ratio(color) * 1000.0) as i32,
        "distance-cie76" => (operand.distance_delta_e_cie76(color) * 1000.0) as i32,
        "distance-ciede2000" => (operand.distance_delta_e_ciede2000(color) * 1000.0) as i32,
        _ => unreachable!("Unknown binary operation"),
    }
}

impl GenericCommand for SortBinaryOperationCommand {
    fn run(&self, out: &mut Output, matches: &ArgMatches, config: &Config) -> Result<()> {
        let binary_operation = matches.value_of("binary-operation").expect("required argument");

        let mut print_spectrum = PrintSpectrum::Yes;

        let operand = ColorArgIterator::from_color_arg(
            config,
            matches.value_of("operand").expect("required argument"),
            &mut print_spectrum,
        )?;

        let mut colors: Vec<Color> = vec![];
        for color in ColorArgIterator::from_args(config, matches.values_of("color"))? {
            colors.push(color?);
        }

        if matches.is_present("unique") {
            colors.sort_by_key(|c| c.to_u32());
            colors.dedup_by_key(|c| c.to_u32());
        }

        colors.sort_by_key(|c| key_function(binary_operation, &operand, c));

        if matches.is_present("reverse") {
            colors.reverse();
        }

        for color in colors {
            out.show_color(config, &color)?;
        }

        Ok(())
    }
}
