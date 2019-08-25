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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimizationTarget {
    Mean,
    Min,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimizationMode {
    Global,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DistanceMetric {
    CIE76,
    CIEDE2000,
}

pub struct SimulatedAnnealing {
    pub colors: Vec<Color>,
    pub temperature: Scalar,
    pub cooling_rate: Scalar,
    pub num_iterations: usize,
    pub opt_target: OptimizationTarget,
    pub opt_mode: OptimizationMode,
    pub distance_metric: DistanceMetric,
}

impl SimulatedAnnealing {
    fn mutual_distance(&self, colors: &[Color]) -> DistanceResult {
        let num_colors = colors.len();

        // The distance to the nearest neighbor for every color
        let mut closest_distances = vec![scalar::MAX; num_colors];

        // The absolute closest distance
        let mut min_closest_distance = scalar::MAX;

        // The indices of the colors that were closest
        let mut closest_pair = (std::usize::MAX, std::usize::MAX);

        for i in 0..num_colors {
            for j in (i + 1)..num_colors {
                let dist = match self.distance_metric {
                    DistanceMetric::CIE76 => colors[i].distance_delta_e_cie76(&colors[j]),
                    DistanceMetric::CIEDE2000 => colors[i].distance_delta_e_ciede2000(&colors[j]),
                };

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

    fn modify_color(&self, color: &mut Color) {
        const STRATEGY: random::strategies::UniformRGB = random::strategies::UniformRGB {};

        match self.opt_mode {
            OptimizationMode::Local => {
                let mut rgb = color.to_rgba();
                Self::modify_channel(&mut rgb.r);
                Self::modify_channel(&mut rgb.g);
                Self::modify_channel(&mut rgb.b);
                *color = Color::from_rgb(rgb.r, rgb.g, rgb.b);
            }
            OptimizationMode::Global => {
                *color = STRATEGY.generate();
            }
        }
    }

    pub fn run<C>(&mut self, mut callback: C)
    where
        C: FnMut(&IterationStatistics),
    {
        let mut result = self.mutual_distance(&self.colors);

        for iter in 0..self.num_iterations {
            let random_index = if self.opt_target == OptimizationTarget::Mean {
                random::<usize>() % self.colors.len()
            } else {
                if random::<bool>() {
                    result.closest_pair.0
                } else {
                    result.closest_pair.1
                }
            };

            let mut new_colors = self.colors.clone();

            self.modify_color(&mut new_colors[random_index]);

            let new_result = self.mutual_distance(&new_colors);

            let (score, new_score) = match self.opt_target {
                OptimizationTarget::Mean => (
                    result.mean_closest_distance,
                    new_result.mean_closest_distance,
                ),
                OptimizationTarget::Min => {
                    (result.min_closest_distance, new_result.min_closest_distance)
                }
            };

            if new_score > score {
                result = new_result;
                self.colors = new_colors;
            } else {
                let bolzmann = Scalar::exp(-(score - new_score) / self.temperature);
                if random::<Scalar>() <= bolzmann {
                    result = new_result;
                    self.colors = new_colors;
                }
            }

            if iter % 5_000 == 0 {
                let statistics = IterationStatistics {
                    iteration: iter,
                    temperature: self.temperature,
                    distance_result: &result,
                    colors: &self.colors,
                };
                callback(&statistics);
            }

            if iter % 1_000 == 0 {
                self.temperature *= self.cooling_rate;
            }
        }
    }
}
