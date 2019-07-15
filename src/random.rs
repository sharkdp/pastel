use rand::prelude::*;

use crate::Color;

pub trait RandomizationStrategy {
    fn generate(&mut self) -> Color;
}

pub struct VividStrategy;

impl RandomizationStrategy for VividStrategy {
    fn generate(&mut self) -> Color {
        let hue = random::<f64>() * 360.0;
        let saturation = 0.2 + 0.6 * random::<f64>();
        let lightness = 0.3 + 0.4 * random::<f64>();

        Color::from_hsl(hue, saturation, lightness)
    }
}
