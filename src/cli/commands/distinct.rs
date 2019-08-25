use crate::commands::prelude::*;
use crate::commands::show::show_color;

use pastel::distinct::{
    IterationStatistics, OptimizationMode, OptimizationTarget, SimulatedAnnealing,
};
use pastel::random::{self, RandomizationStrategy};

pub struct DistinctCommand;

fn print_iteration(out: &mut dyn Write, brush: &Brush, stats: &IterationStatistics) -> Result<()> {
    let result = stats.distance_result;
    write!(
        out,
        "[{:10.}] D_mean = {:<6.2}; D_min = {:<6.2}; T = {:.6} ",
        stats.iteration,
        result.mean_closest_distance,
        result.min_closest_distance,
        stats.temperature
    )?;
    print_colors(out, brush, stats.colors, Some(result.closest_pair))?;
    Ok(())
}

fn print_colors(
    out: &mut dyn Write,
    brush: &Brush,
    colors: &[Color],
    closest_pair: Option<(usize, usize)>,
) -> Result<()> {
    let mut ci = 0;
    for c in colors.iter() {
        let tc = c.text_color();
        let mut style = tc.ansi_style();
        style.on(c);

        if let Some(pair) = closest_pair {
            if pair.0 == ci || pair.1 == ci {
                style.bold(true);
                style.underline(true);
            }
        }

        print!(
            "{} ",
            brush.paint(format!("{}", c.to_rgb_hex_string(false)), style)
        );

        ci += 1;
    }
    writeln!(out, "")?;
    Ok(())
}

impl GenericCommand for DistinctCommand {
    fn run(&self, out: &mut dyn Write, matches: &ArgMatches, config: &Config) -> Result<()> {
        let count = matches.value_of("number").expect("required argument");
        let count = count
            .parse::<usize>()
            .map_err(|_| PastelError::CouldNotParseNumber(count.into()))?;

        let mut colors = Vec::new();
        for _ in 0..count {
            colors.push(random::strategies::UniformRGB.generate());
        }

        let mut annealing = SimulatedAnnealing { colors };

        annealing.run(
            |stats| {
                print_iteration(out, &config.brush, stats).ok();
            },
            200_000,
            3.0,
            0.95,
            OptimizationTarget::Mean,
            OptimizationMode::Global,
        );
        annealing.run(
            |stats| {
                print_iteration(out, &config.brush, stats).ok();
            },
            1_000_000,
            0.5,
            0.99,
            OptimizationTarget::Min,
            OptimizationMode::Local,
        );

        for color in annealing.colors {
            show_color(out, config, &color)?;
        }

        Ok(())
    }
}
