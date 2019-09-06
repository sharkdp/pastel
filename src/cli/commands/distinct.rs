use crate::commands::prelude::*;
use pastel::distinct::{
    DistanceMetric, distinct_colors,
};

pub struct DistinctCommand;

impl GenericCommand for DistinctCommand {
    fn run(&self, out: &mut Output, matches: &ArgMatches, config: &Config) -> Result<()> {
        let count = matches.value_of("number").expect("required argument");
        let count = count
            .parse::<usize>()
            .map_err(|_| PastelError::CouldNotParseNumber(count.into()))?;

        let distance_metric = match matches.value_of("metric").expect("required argument") {
            "CIE76" => DistanceMetric::CIE76,
            "CIEDE2000" => DistanceMetric::CIEDE2000,
            _ => unreachable!("Unknown distance metric"),
        };

        let (colors, distance_result) = distinct_colors(
            count,
            distance_metric,
            matches.is_present("verbose"),
            !matches.is_present("print-minimal-distance")
        )?;

        if matches.is_present("print-minimal-distance") {
            writeln!(out.handle, "{:.3}", distance_result.min_closest_distance)?;
        } else {
            for color in colors {
                out.show_color(config, &color)?;
            }
        }

        Ok(())
    }
}
