use core::f64 as scalar;

use rand::prelude::*;

use crate::random::{self, RandomizationStrategy};
use crate::Color;

type Scalar = f64;

pub struct DistanceResult {
    /// The closest distance between any two colors
    pub min_closest_distance: Scalar,

    /// The average over all nearest-neighbor distances
    pub mean_closest_distance: Scalar,

    /// Indices of the colors that were closest to each other
    pub closest_pair: (usize, usize),
}

pub struct IterationStatistics<'a> {
    pub iteration: usize,
    pub temperature: Scalar,
    pub distance_result: &'a DistanceResult,
    pub colors: &'a [Color],
}

pub fn mutual_distance(colors: &[Color]) -> DistanceResult {
    let num_colors = colors.len();

    // The distance to the nearest neighbor for every color
    let mut closest_distances = vec![scalar::MAX; num_colors];

    // The absolute closest distance
    let mut min_closest_distance = scalar::MAX;

    // The indices of the colors that were closest
    let mut closest_pair = (std::usize::MAX, std::usize::MAX);

    for i in 0..num_colors {
        for j in (i + 1)..num_colors {
            let dist = colors[i].distance_delta_e_ciede2000(&colors[j]);

            if dist < min_closest_distance {
                min_closest_distance = dist;
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

    DistanceResult {
        min_closest_distance,
        mean_closest_distance,
        closest_pair,
    }
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
    C: FnMut(&IterationStatistics),
{
    let mut temperature = initial_temp;

    let mut result = mutual_distance(colors);

    for iter in 0..num_iter {
        let random_index = if optimize_mean || only_small_modifications {
            random::<usize>() % colors.len()
        } else {
            if random::<bool>() {
                result.closest_pair.0
            } else {
                result.closest_pair.1
            }
        };

        let mut new_colors = colors.clone();

        modify_color(&mut new_colors[random_index], only_small_modifications);

        let new_result = mutual_distance(&new_colors);

        let score = if optimize_mean {
            result.mean_closest_distance
        } else {
            result.min_closest_distance
        };
        let new_score = if optimize_mean {
            new_result.mean_closest_distance
        } else {
            new_result.min_closest_distance
        };

        if new_score > score {
            result = new_result;
            *colors = new_colors;
        } else {
            let bolzmann = Scalar::exp(-(score - new_score) / temperature);
            if random::<Scalar>() <= bolzmann {
                result = new_result;
                *colors = new_colors;
            }
        }

        if iter % 5_000 == 0 {
            let statistics = IterationStatistics {
                iteration: iter,
                temperature,
                distance_result: &result,
                colors,
            };
            callback(&statistics);
        }

        if iter % 1_000 == 0 {
            temperature *= cooling_rate;
        }
    }
}
