use std::env;

use rand::prelude::*;

use core::f64 as scalar;
use pastel::ansi::{Brush, ToAnsiStyle};
use pastel::random::{self, RandomizationStrategy};
use pastel::Color;

type Scalar = f64;

fn mutual_distance(colors: &[Color]) -> (Scalar, Scalar, (usize, usize)) {
    let num = colors.len();

    let mut distances = vec![vec![0.0; num]; num];
    let mut min_closest_dist = scalar::MAX;

    let mut pair = (0, 0);

    for i in 0..num {
        for j in (i + 1)..num {
            let dist = colors[i].distance_delta_e_cie76(&colors[j]);
            if dist < min_closest_dist {
                min_closest_dist = dist;
                pair = (i, j);
            }
            distances[i][j] = dist;
            distances[j][i] = dist;
        }
    }

    // Find the distance to the nearest neighbor for each color
    let mut closest_distances = vec![scalar::MAX; num];
    for i in 0..num {
        for j in 0..num {
            if i != j && distances[i][j] < closest_distances[i] {
                closest_distances[i] = distances[i][j];
            }
        }
    }

    let mut mean_closest_distance = 0.0;
    for dist in closest_distances {
        mean_closest_distance += dist;
    }
    mean_closest_distance /= num as Scalar;

    (min_closest_dist, mean_closest_distance, pair)
}

fn modify_channel(c: &mut u8) {
    if random::<bool>() {
        *c = c.saturating_add(random::<u8>() % 10);
    } else {
        *c = c.saturating_sub(random::<u8>() % 10);
    }
}

fn modify_color(color: &mut Color, only_small_modifications: bool) {
    const STRATEGY: random::strategies::UniformRGB = random::strategies::UniformRGB {};

    if only_small_modifications {
        let mut rgb = color.to_rgba();
        modify_channel(&mut rgb.r);
        modify_channel(&mut rgb.g);
        modify_channel(&mut rgb.b);
        *color = Color::from_rgb(rgb.r, rgb.g, rgb.b);
    } else {
        *color = STRATEGY.generate();
    }
}

fn print_colors(brush: &Brush, colors: &[Color]) {
    let (_, _, pair) = mutual_distance(colors);
    let mut ci = 0;
    for c in colors.iter() {
        let tc = c.text_color();
        let mut style = tc.ansi_style();
        style.on(c);

        if pair.0 == ci || pair.1 == ci {
            style.bold(true);
            style.underline(true);
        }

        print!(
            "{} ",
            brush.paint(format!("{}", c.to_rgb_hex_string(false)), style)
        );

        ci += 1;
    }
    println!("");
}

fn annealing(
    brush: &Brush,
    colors: &mut Vec<Color>,
    num_iter: usize,
    initial_temp: f64,
    cooling_rate: f64,
    optimize_mean: bool,
    only_small_modifications: bool,
) {
    let mut temperature = initial_temp;

    // let mut strategy = random::strategies::UniformRGB {};

    let (mut min_closest_distance, mut mean_closest_distance, mut pair) = mutual_distance(colors);

    for iter in 0..num_iter {
        let random_index = if optimize_mean || only_small_modifications {
            random::<usize>() % colors.len()
        } else {
            if random::<bool>() {
                pair.0
            } else {
                pair.1
            }
        };
        // let random_index = random::<usize>() % colors.len();

        let mut new_colors = colors.clone();

        modify_color(&mut new_colors[random_index], only_small_modifications);

        let (new_min_dist, new_mean_dist, new_pair) = mutual_distance(&new_colors);

        let score = if optimize_mean {
            mean_closest_distance
        } else {
            min_closest_distance
        };
        let new_score = if optimize_mean {
            new_mean_dist
        } else {
            new_min_dist
        };

        if new_score > score {
            min_closest_distance = new_min_dist;
            mean_closest_distance = new_mean_dist;
            pair = new_pair;
            *colors = new_colors;
        } else {
            let bolzmann = Scalar::exp(-(score - new_score) / temperature);
            if random::<Scalar>() <= bolzmann {
                min_closest_distance = new_min_dist;
                mean_closest_distance = new_mean_dist;
                pair = new_pair;
                *colors = new_colors;
            }
        }

        if iter % 5_000 == 0 {
            // colors.sort_by_key(|c| (c.to_lch().h * 100.0) as i32);
            print!(
                "[{:10.}] D_mean = {:<6.2}; D_min = {:<6.2}; T = {:.6} ",
                iter, mean_closest_distance, min_closest_distance, temperature
            );
            print_colors(brush, &colors);
        }

        if iter % 1_000 == 0 {
            temperature *= cooling_rate;
        }
    }
}

fn main() {
    let n = env::args()
        .nth(1)
        .map(|n| n.parse::<usize>().unwrap())
        .unwrap_or(10);

    let mut colors = Vec::new();
    for _ in 0..n {
        colors.push(random::strategies::UniformRGB.generate());
    }
    let brush = Brush::from_environment();

    annealing(&brush, &mut colors, 200_000, 3.0, 0.95, true, false);
    annealing(&brush, &mut colors, 1_000_000, 0.5, 0.99, false, true);

    println!();
    println!("Sorted by L*:");
    colors.sort_by_key(|c| (c.to_lab().l * 100.0) as i32);
    print_colors(&brush, &colors);
    println!("Sorted by a*:");
    colors.sort_by_key(|c| (c.to_lab().a * 100.0) as i32);
    print_colors(&brush, &colors);
    println!("Sorted by b*:");
    colors.sort_by_key(|c| (c.to_lab().b * 100.0) as i32);
    print_colors(&brush, &colors);
}
