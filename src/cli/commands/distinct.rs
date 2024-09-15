use std::io::{self, Write};

use crate::commands::prelude::*;

use pastel::ansi::Stream;
use pastel::distinct::{self, DistanceMetric, IterationStatistics};
use pastel::{Fraction, HSLA};

pub struct DistinctCommand;

fn print_iteration(out: &mut dyn Write, brush: Brush, stats: &IterationStatistics) -> Result<()> {
    let result = stats.distance_result;
    write!(
        out,
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
    out: &mut dyn Write,
    brush: Brush,
    colors: &[Color],
    closest_pair: Option<(usize, usize)>,
) -> Result<()> {
    for (ci, c) in colors.iter().enumerate() {
        let tc = c.text_color();
        let mut style = tc.ansi_style();
        style.on(c);

        if let Some(pair) = closest_pair {
            if pair.0 == ci || pair.1 == ci {
                style.bold(true);
                style.underline(true);
            }
        }

        write!(out, "{} ", brush.paint(c.to_rgb_hex_string(false), style))?;
    }
    writeln!(out)?;
    Ok(())
}

fn blue_red_yellow(f: f64) -> Color {
    let blue = Color::from_rgb(0, 0, 120);
    let red = Color::from_rgb(224, 0, 119);
    let yellow = Color::from_rgb(255, 255, 0);

    if f < 0.5 {
        blue.mix::<HSLA>(&red, Fraction::from(2.0 * f))
    } else {
        red.mix::<HSLA>(&yellow, Fraction::from(2.0 * (f - 0.5)))
    }
}

fn print_distance_matrix(
    out: &mut dyn Write,
    brush: Brush,
    colors: &[Color],
    metric: DistanceMetric,
) -> Result<()> {
    let count = colors.len();

    let distance = |c1: &Color, c2: &Color| match metric {
        DistanceMetric::CIE76 => c1.distance_delta_e_cie76(c2),
        DistanceMetric::CIEDE2000 => c1.distance_delta_e_ciede2000(c2),
    };

    let mut min = f64::MAX;
    let mut max = 0.0;
    for i in 0..count {
        for j in 0..count {
            if i != j {
                let dist = distance(&colors[i], &colors[j]);
                if dist < min {
                    min = dist;
                }
                if dist > max {
                    max = dist;
                }
            }
        }
    }

    let color_to_string = |c: &Color| -> String {
        let tc = c.text_color();
        let mut style = tc.ansi_style();
        style.on(c);
        brush.paint(c.to_rgb_hex_string(false), style)
    };

    write!(out, "\n\n{:6}  ", "")?;
    for c in colors {
        write!(out, "{} ", color_to_string(c))?;
    }
    writeln!(out, "\n")?;

    for c1 in colors {
        write!(out, "{}  ", color_to_string(c1))?;
        for c2 in colors {
            if c1 == c2 {
                write!(out, "{:6} ", "")?;
            } else {
                let dist = distance(c1, c2);

                let magnitude = (dist - min) / (max - min);
                let magnitude = 1.0 - magnitude.powf(0.3);

                let bg = blue_red_yellow(magnitude);
                let mut style = bg.text_color().ansi_style();
                style.on(bg);

                write!(out, "{} ", brush.paint(format!("{:6.2}", dist), style))?;
            }
        }
        writeln!(out)?;
    }
    writeln!(out, "\n")?;

    Ok(())
}

impl GenericCommand for DistinctCommand {
    fn run(&self, out: &mut Output, matches: &ArgMatches, config: &Config) -> Result<()> {
        let stderr = io::stderr();
        let mut stderr_lock = stderr.lock();
        let brush_stderr = Brush::from_environment(Stream::Stderr)?;
        let verbose_output = matches.is_present("verbose");

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

        let fixed_colors = match matches.values_of("color") {
            None => vec![],
            Some(positionals) => {
                ColorArgIterator::FromPositionalArguments(config, positionals, PrintSpectrum::Yes)
                    .collect::<Result<Vec<_>>>()?
            }
        };

        let num_fixed_colors = fixed_colors.len();
        if num_fixed_colors > count {
            return Err(PastelError::DistinctColorFixedColorsCannotBeMoreThanCount);
        }

        let mut callback: Box<dyn FnMut(&IterationStatistics)> = if verbose_output {
            Box::new(|stats: &IterationStatistics| {
                print_iteration(&mut stderr_lock, brush_stderr, stats).ok();
            })
        } else {
            Box::new(|_: &IterationStatistics| {})
        };

        let (mut colors, distance_result) =
            distinct::distinct_colors(count, distance_metric, fixed_colors, callback.as_mut());

        if matches.is_present("print-minimal-distance") {
            writeln!(out.handle, "{:.3}", distance_result.min_closest_distance)?;
        } else {
            distinct::rearrange_sequence(&mut colors, distance_metric);

            if verbose_output {
                print_distance_matrix(&mut stderr.lock(), brush_stderr, &colors, distance_metric)?;
            }

            for color in colors {
                out.show_color(config, &color)?;
            }
        }

        Ok(())
    }
}
