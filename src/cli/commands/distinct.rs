use std::io;

use crate::commands::prelude::*;
use crate::output::Output;

use pastel::ansi::Stream;
use pastel::distinct::{
    self, DistanceMetric, IterationStatistics, OptimizationMode, OptimizationTarget,
    SimulatedAnnealing, SimulationParameters,
};
use pastel::random::{self, RandomizationStrategy};

pub struct DistinctCommand;

fn print_iteration(out: &mut Output, brush: &Brush, stats: &IterationStatistics) -> Result<()> {
    let result = stats.distance_result;
    write!(
        out.handle,
        "[{:10.}] D_mean = {:<6.2}; D_min = {:<6.2}; T = {:.6} ",
        stats.iteration,
        result.mean_closest_distance,
        result.min_closest_distance,
        stats.temperature
    )?;
    print_colors(out, brush, &stats.colors, Some(result.closest_pair))?;
    Ok(())
}

fn print_colors(
    out: &mut Output,
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

        write!(
            out.handle,
            "{} ",
            brush.paint(format!("{}", c.to_rgb_hex_string(false)), style)
        )?;

        ci += 1;
    }
    writeln!(out.handle, "")?;
    Ok(())
}

impl GenericCommand for DistinctCommand {
    fn run(&self, out: &mut Output, matches: &ArgMatches, config: &Config) -> Result<()> {
        let stderr = io::stderr();
        let brush_stderr = Brush::from_environment(Stream::Stderr);

        let count = matches.value_of("number").expect("required argument");
        let count = count
            .parse::<usize>()
            .map_err(|_| PastelError::CouldNotParseNumber(count.into()))?;

        if count < 2 {
            return Err(PastelError::DistinctColorCountMustBeLargerThanOne);
        }

        let distance_metric = match matches.value_of("metric").expect("required argument") {
            "CIE76" => DistanceMetric::CIE76,
            "CIEDE2000" => DistanceMetric::CIEDE2000,
            _ => unreachable!("Unknown distance metric"),
        };

        let mut colors = Vec::new();
        for _ in 0..count {
            colors.push(random::strategies::UniformRGB.generate());
        }

        let mut annealing = SimulatedAnnealing::new(
            &colors,
            SimulationParameters {
                initial_temperature: 3.0,
                cooling_rate: 0.95,
                num_iterations: 100_000,
                opt_target: OptimizationTarget::Mean,
                opt_mode: OptimizationMode::Global,
                distance_metric,
            },
        );

        let mut callback: Box<dyn FnMut(&IterationStatistics)> = if matches.is_present("verbose") {
            Box::new(|stats: &IterationStatistics| {
                print_iteration(&mut Output::new(&mut stderr.lock()), &brush_stderr, stats).ok();
            })
        } else {
            Box::new(|_: &IterationStatistics| {})
        };

        annealing.run(callback.as_mut());

        annealing.parameters.initial_temperature = 0.5;
        annealing.parameters.cooling_rate = 0.98;
        annealing.parameters.num_iterations = 200_000;
        annealing.parameters.opt_target = OptimizationTarget::Min;
        annealing.parameters.opt_mode = OptimizationMode::Local;

        let result = annealing.run(callback.as_mut());

        if matches.is_present("print-minimal-distance") {
            writeln!(out.handle, "{:.3}", result.min_closest_distance)?;
        } else {
            let mut colors = annealing.get_colors();
            distinct::rearrange_sequence(&mut colors, distance_metric);

            for color in colors {
                out.show_color(config, &color)?;
            }
        }

        Ok(())
    }
}
