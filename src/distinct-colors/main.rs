use std::env;

use rand::prelude::*;

use pastel::ansi::{Brush, ToAnsiStyle};
use pastel::random::{self, RandomizationStrategy};
use pastel::Color;

fn mutual_distance(colors: &[Color]) -> f64 {
    let num = colors.len();

    let mut min_dist = 1000.0;

    for i in 0..num {
        for j in (i + 1)..num {
            let dist = colors[i].distance(&colors[j]);
            if dist < min_dist {
                min_dist = dist;
            }
        }
    }

    min_dist
}

fn annealing(brush: &Brush, colors: &mut Vec<Color>) {
    let mut strategy = random::strategies::UniformRGB {};

    let mut min_distance = mutual_distance(colors);
    for iter in 0..10_000_000 {
        let random_index = random::<usize>() % colors.len();
        let new_color = strategy.generate();
        let mut new_set = colors.clone();
        new_set[random_index] = new_color;
        let new_min_dist = mutual_distance(&new_set);

        if new_min_dist > min_distance {
            min_distance = new_min_dist;
            *colors = new_set;
        } else if new_min_dist == min_distance {
            *colors = new_set;
        }

        if iter % 100_000 == 0 {
            println!("[Iteration {}] min. distance: {:.2}", iter, min_distance);
            for c in colors.iter() {
                let tc = c.text_color();
                let mut style = tc.ansi_style();
                style.on(c);
                print!(
                    "{} ",
                    brush.paint(format!("{}", c.to_rgb_hex_string()), style)
                );
            }
            println!();
        }
    }
}

fn main() {
    let n = env::args()
        .nth(1)
        .map(|n| n.parse::<usize>().unwrap())
        .unwrap_or(10);

    let mut colors = Vec::new();
    colors.resize(n, Color::black());

    let brush = Brush::from_environment();
    annealing(&brush, &mut colors);
    let min_dist = mutual_distance(&colors);
    println!("min dist: {:.2}", min_dist);
}
