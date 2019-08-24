use core::f64 as scalar;

use rand::prelude::*;

use crate::random::{self, RandomizationStrategy};
use crate::Color;

type Scalar = f64;

pub fn mutual_distance(colors: &[Color]) -> (Scalar, Scalar, (usize, usize)) {
    let num_colors = colors.len();

    // The distance to the nearest neighbor for every color
    let mut closest_distances = vec![scalar::MAX; num_colors];

    // The absolute closest distance
    let mut min_closest_dist = scalar::MAX;

    // The indices of the colors that were closest
    let mut closest_pair = (std::usize::MAX, std::usize::MAX);

    for i in 0..num_colors {
        for j in (i + 1)..num_colors {
            let dist = colors[i].distance_delta_e_ciede2000(&colors[j]);

            if dist < min_closest_dist {
                min_closest_dist = dist;
                closest_pair = (i, j);
            }

            if dist < closest_distances[i] {
                closest_distances[i] = dist;
            }

            if dist < closest_distances[j] {
                closest_distances[j] = dist;
            }
        }
    }

    let mut mean_closest_distance = 0.0;
    for dist in closest_distances {
        mean_closest_distance += dist;
    }
    mean_closest_distance /= num_colors as Scalar;

    (min_closest_dist, mean_closest_distance, closest_pair)
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

pub fn annealing<C>(
    mut callback: C,
    colors: &mut Vec<Color>,
    num_iter: usize,
    initial_temp: Scalar,
    cooling_rate: Scalar,
    optimize_mean: bool,
    only_small_modifications: bool,
) where
    C: FnMut(&[Color]),
{
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
            // print_colors(brush, &colors);
            callback(&colors);
        }

        if iter % 1_000 == 0 {
            temperature *= cooling_rate;
        }
    }
}
