use std::env;

use pastel::ansi::{Brush, ToAnsiStyle};
use pastel::distinct::{annealing, IterationStatistics};
use pastel::random::{self, RandomizationStrategy};
use pastel::Color;

fn print_iteration(brush: &Brush, stats: &IterationStatistics) {
    let result = stats.distance_result;
    print!(
        "[{:10.}] D_mean = {:<6.2}; D_min = {:<6.2}; T = {:.6} ",
        stats.iteration,
        result.mean_closest_distance,
        result.min_closest_distance,
        stats.temperature
    );
    print_colors(brush, stats.colors, Some(result.closest_pair));
}

fn print_colors(brush: &Brush, colors: &[Color], closest_pair: Option<(usize, usize)>) {
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
    println!("");
}

fn main() {
    let n = env::args()
        .nth(1)
        .map(|n| n.parse::<usize>().unwrap())
        .unwrap_or(10);

    let brush = Brush::from_environment();

    let mut colors = Vec::new();
    for _ in 0..n {
        colors.push(random::strategies::UniformRGB.generate());
    }

    let callback = |stats: &IterationStatistics| {
        print_iteration(&brush, stats);
    };

    annealing(callback, &mut colors, 200_000, 3.0, 0.95, true, false);
    annealing(callback, &mut colors, 1_000_000, 0.5, 0.99, false, true);

    println!();
    println!("Sorted by L*:");
    colors.sort_by_key(|c| (c.to_lab().l * 100.0) as i32);
    print_colors(&brush, &colors, None);
    println!("Sorted by a*:");
    colors.sort_by_key(|c| (c.to_lab().a * 100.0) as i32);
    print_colors(&brush, &colors, None);
    println!("Sorted by b*:");
    colors.sort_by_key(|c| (c.to_lab().b * 100.0) as i32);
    print_colors(&brush, &colors, None);
}
