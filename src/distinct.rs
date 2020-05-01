use core::f64 as scalar;

use rand::prelude::*;

use crate::delta_e;
use crate::random::{self, RandomizationStrategy};
use crate::{Color, Lab};

type Scalar = f64;

#[derive(Clone)]
pub struct DistanceResult {
    /// The closest distance between any two colors
    pub min_closest_distance: Scalar,

    /// The average over all nearest-neighbor distances
    pub mean_closest_distance: Scalar,

    /// Indices of the colors that were closest to each other
    pub closest_pair: (usize, usize),

    /// The closest distance and the index of the nearest neighbor
    pub closest_distances: Vec<(Scalar, usize)>,

    pub distance_metric: DistanceMetric,

    /// The number of colors that are fixed and cannot be changed. The actual colors are the first
    /// `num_fixed_colors` elements in the `colors` array.
    pub num_fixed_colors: usize,
}

pub struct IterationStatistics<'a> {
    pub iteration: usize,
    pub temperature: Scalar,
    pub distance_result: &'a DistanceResult,
    pub colors: Vec<Color>,
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

pub struct SimulationParameters {
    pub initial_temperature: Scalar,
    pub cooling_rate: Scalar,
    pub num_iterations: usize,
    pub opt_target: OptimizationTarget,
    pub opt_mode: OptimizationMode,
    pub distance_metric: DistanceMetric,
    pub num_fixed_colors: usize,
}

pub struct SimulatedAnnealing<R: Rng> {
    colors: Vec<(Color, Lab)>,
    temperature: Scalar,
    pub parameters: SimulationParameters,
    rng: R,
}

impl SimulatedAnnealing<ThreadRng> {
    pub fn new(initial_colors: &[Color], parameters: SimulationParameters) -> Self {
        Self::with_rng(initial_colors, parameters, thread_rng())
    }
}

impl<R: Rng> SimulatedAnnealing<R> {
    pub fn with_rng(initial_colors: &[Color], parameters: SimulationParameters, rng: R) -> Self {
        let colors = initial_colors
            .iter()
            .map(|c| (c.clone(), c.to_lab()))
            .collect();

        SimulatedAnnealing {
            colors,
            temperature: parameters.initial_temperature,
            parameters,
            rng,
        }
    }
}

impl<R: Rng> SimulatedAnnealing<R> {
    pub fn get_colors(&self) -> Vec<Color> {
        self.colors.iter().map(|(c, _)| c.clone()).collect()
    }

    fn modify_channel(&mut self, c: &mut u8) {
        if self.rng.gen::<bool>() {
            *c = c.saturating_add(self.rng.gen::<u8>() % 10);
        } else {
            *c = c.saturating_sub(self.rng.gen::<u8>() % 10);
        }
    }

    fn modify_color(&mut self, color: &mut (Color, Lab)) {
        const STRATEGY: random::strategies::UniformRGB = random::strategies::UniformRGB {};

        match self.parameters.opt_mode {
            OptimizationMode::Local => {
                let mut rgb = color.0.to_rgba();
                self.modify_channel(&mut rgb.r);
                self.modify_channel(&mut rgb.g);
                self.modify_channel(&mut rgb.b);
                color.0 = Color::from_rgb(rgb.r, rgb.g, rgb.b);
            }
            OptimizationMode::Global => {
                color.0 = STRATEGY.generate_with(&mut self.rng);
            }
        }
        color.1 = color.0.to_lab();
    }

    pub fn run(&mut self, callback: &mut dyn FnMut(&IterationStatistics)) -> DistanceResult {
        self.temperature = self.parameters.initial_temperature;

        let mut result = DistanceResult::new(
            &self.colors,
            self.parameters.distance_metric,
            self.parameters.num_fixed_colors,
        );

        if self.parameters.num_fixed_colors == self.colors.len() {
            return result;
        }

        for iter in 0..self.parameters.num_iterations {
            let random_index = if self.parameters.opt_target == OptimizationTarget::Mean {
                self.rng
                    .gen_range(self.parameters.num_fixed_colors, self.colors.len())
            } else {
                // first check if any of the colors cannot change, if that's the case just return
                // the other color. Note that the closest_pair cannot contain only fixed colors.
                if result.closest_pair.0 < self.parameters.num_fixed_colors {
                    result.closest_pair.1
                } else if result.closest_pair.1 < self.parameters.num_fixed_colors {
                    result.closest_pair.0
                } else if self.rng.gen() {
                    result.closest_pair.0
                } else {
                    result.closest_pair.1
                }
            };

            debug_assert!(
                random_index >= self.parameters.num_fixed_colors,
                "cannot change fixed color"
            );

            let mut new_colors = self.colors.clone();

            self.modify_color(&mut new_colors[random_index]);

            let new_result = result.update(&new_colors, random_index);

            let (score, new_score) = match self.parameters.opt_target {
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
                if self.rng.gen::<Scalar>() <= bolzmann {
                    result = new_result;
                    self.colors = new_colors;
                }
            }

            if iter % 5_000 == 0 {
                let statistics = IterationStatistics {
                    iteration: iter,
                    temperature: self.temperature,
                    distance_result: &result,
                    colors: self.get_colors(),
                };
                callback(&statistics);
            }

            if iter % 1_000 == 0 {
                self.temperature *= self.parameters.cooling_rate;
            }
        }

        result
    }
}

/// Re-arrange the sequence of colors such that the minimal difference between a given color and
/// any of its predecessors is maximized.
///
/// Note: this is only a heuristic and will not yield optimal results (especially at the end of
/// the sequence).
///
/// See: https://en.wikipedia.org/wiki/Farthest-first_traversal
pub fn rearrange_sequence(colors: &mut Vec<Color>, metric: DistanceMetric) {
    let distance = |c1: &Color, c2: &Color| match metric {
        DistanceMetric::CIE76 => c1.distance_delta_e_cie76(c2),
        DistanceMetric::CIEDE2000 => c1.distance_delta_e_ciede2000(c2),
    };

    // vector where the i-th element contains the minimum distance to the colors from 0 to i-1.
    let mut min_distances = vec![i32::max_value(); colors.len()];

    for i in 1..colors.len() {
        let mut max_i = colors.len();
        let mut max_d = i32::min_value();

        for j in i..colors.len() {
            min_distances[j] =
                min_distances[j].min((distance(&colors[j], &colors[i - 1]) * 1000.0) as i32);

            if min_distances[j] > max_d {
                max_i = j;
                max_d = min_distances[j];
            }
        }

        colors.swap(i, max_i);
        min_distances.swap(i, max_i);
    }
}

pub fn distinct_colors(
    count: usize,
    distance_metric: DistanceMetric,
    fixed_colors: Vec<Color>,
    callback: &mut dyn FnMut(&IterationStatistics),
) -> (Vec<Color>, DistanceResult) {
    assert!(count > 1);
    assert!(fixed_colors.len() <= count);

    let num_fixed_colors = fixed_colors.len();
    let mut colors = fixed_colors;

    for _ in num_fixed_colors..count {
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
            num_fixed_colors,
        },
    );

    annealing.run(callback);

    annealing.parameters.initial_temperature = 0.5;
    annealing.parameters.cooling_rate = 0.98;
    annealing.parameters.num_iterations = 200_000;
    annealing.parameters.opt_target = OptimizationTarget::Min;
    annealing.parameters.opt_mode = OptimizationMode::Local;

    let result = annealing.run(callback);

    (annealing.get_colors(), result)
}

impl DistanceResult {
    fn new(
        colors: &[(Color, Lab)],
        distance_metric: DistanceMetric,
        num_fixed_colors: usize,
    ) -> Self {
        let mut result = DistanceResult {
            closest_distances: vec![(scalar::MAX, std::usize::MAX); colors.len()],
            closest_pair: (std::usize::MAX, std::usize::MAX),
            mean_closest_distance: 0.0,
            min_closest_distance: scalar::MAX,
            distance_metric,
            num_fixed_colors,
        };

        for i in 0..colors.len() {
            result.update_distances(colors, i, false);
        }
        result.update_totals();

        result
    }

    fn update(&self, colors: &[(Color, Lab)], changed_color: usize) -> Self {
        let mut result = self.clone();
        result.update_distances(colors, changed_color, true);
        result.update_totals();
        result
    }

    fn update_distances(&mut self, colors: &[(Color, Lab)], color: usize, changed: bool) {
        self.closest_distances[color] = (scalar::MAX, std::usize::MAX);

        // we need to recalculate distances for nodes where the previous min dist was with
        // changed_color but it's not anymore (potentially).
        let mut to_recalc = Vec::with_capacity(colors.len());

        for (i, c) in colors.iter().enumerate() {
            if i == color {
                continue;
            }

            let dist = self.distance(c, &colors[color]);

            if dist < self.closest_distances[i].0 {
                self.closest_distances[i] = (dist, color);
            } else if changed && self.closest_distances[i].1 == color {
                // changed_color was the best before, but unfortunately we cannot say it now for
                // sure because the distance between the two increased. Play it safe and just
                // recalculate its distances.
                to_recalc.push(i);
            }

            if dist < self.closest_distances[color].0 {
                self.closest_distances[color] = (dist, i);
            }
        }

        for i in to_recalc {
            self.update_distances(colors, i, false);
        }
    }

    fn update_totals(&mut self) {
        self.mean_closest_distance = 0.0;
        self.min_closest_distance = scalar::MAX;

        let mut closest_pair_set = false;

        for (i, (dist, closest_i)) in self.closest_distances.iter().enumerate() {
            if i < self.num_fixed_colors && *closest_i < self.num_fixed_colors {
                continue;
            }

            self.mean_closest_distance += *dist;

            // the closest pair must ignore pairs of fixed colors because we cannot change them. On
            // the other hand we can consider pairs with at least one non fixed color because we
            // can change that.
            if (i >= self.num_fixed_colors || *closest_i >= self.num_fixed_colors)
                && (*dist < self.min_closest_distance || !closest_pair_set)
            {
                self.closest_pair = (i, *closest_i);
                closest_pair_set = true;
            }

            self.min_closest_distance = self.min_closest_distance.min(*dist);
        }

        self.mean_closest_distance /=
            (self.closest_distances.len() - self.num_fixed_colors) as Scalar;
    }

    fn distance(&self, a: &(Color, Lab), b: &(Color, Lab)) -> Scalar {
        match self.distance_metric {
            DistanceMetric::CIE76 => delta_e::cie76(&a.1, &b.1),
            DistanceMetric::CIEDE2000 => delta_e::ciede2000(&a.1, &b.1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        rearrange_sequence, DistanceMetric, OptimizationMode, OptimizationTarget,
        SimulatedAnnealing, SimulationParameters,
    };
    use crate::Color;

    use rand::prelude::*;
    use rand_xoshiro::Xoshiro256StarStar;

    #[test]
    fn test_rearrange_sequence() {
        let mut colors = vec![
            Color::white(),
            Color::graytone(0.25),
            Color::graytone(0.5),
            Color::graytone(0.8),
            Color::black(),
        ];

        rearrange_sequence(&mut colors, DistanceMetric::CIE76);

        assert_eq!(
            colors,
            vec![
                Color::white(),
                Color::black(),
                Color::graytone(0.5),
                Color::graytone(0.25),
                Color::graytone(0.8),
            ]
        );
    }

    #[test]
    fn test_distinct_all_fixed_colors() {
        let colors = [Color::red(), Color::olive(), Color::yellow()];

        let mut sim = SimulatedAnnealing::with_rng(
            &colors,
            SimulationParameters {
                initial_temperature: 3.0,
                cooling_rate: 0.95,
                num_iterations: 100,
                opt_target: OptimizationTarget::Min,
                opt_mode: OptimizationMode::Local,
                distance_metric: DistanceMetric::CIE76,
                num_fixed_colors: 3,
            },
            Xoshiro256StarStar::seed_from_u64(21),
        );
        sim.run(&mut |_| {});

        assert_eq!(
            sim.get_colors(),
            vec![Color::red(), Color::olive(), Color::yellow()]
        );
    }

    #[test]
    fn test_distinct_2_fixed_colors() {
        let colors = [Color::red(), Color::yellow()];

        let mut sim = SimulatedAnnealing::with_rng(
            &colors,
            SimulationParameters {
                initial_temperature: 3.0,
                cooling_rate: 0.95,
                num_iterations: 100,
                opt_target: OptimizationTarget::Min,
                opt_mode: OptimizationMode::Local,
                distance_metric: DistanceMetric::CIE76,
                num_fixed_colors: 1,
            },
            Xoshiro256StarStar::seed_from_u64(42),
        );
        sim.run(&mut |_| {});

        assert_eq!(sim.get_colors()[0], Color::red());
    }
}
