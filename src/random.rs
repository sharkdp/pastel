use crate::Color;

use rand::{prelude::*, rng};

pub trait RandomizationStrategy {
    fn generate(&mut self) -> Color {
        self.generate_with(&mut rng())
    }

    fn generate_with(&mut self, r: &mut dyn RngCore) -> Color;
}

pub mod strategies {
    use super::RandomizationStrategy;
    use crate::Color;

    use rand::prelude::*;

    pub struct Vivid;

    impl RandomizationStrategy for Vivid {
        fn generate_with(&mut self, rng: &mut dyn RngCore) -> Color {
            let hue = rng.random::<f64>() * 360.0;
            let saturation = 0.2 + 0.6 * rng.random::<f64>();
            let lightness = 0.3 + 0.4 * rng.random::<f64>();

            Color::from_hsl(hue, saturation, lightness)
        }
    }

    pub struct UniformRGB;

    impl RandomizationStrategy for UniformRGB {
        fn generate_with(&mut self, rng: &mut dyn RngCore) -> Color {
            Color::from_rgb(rng.random::<u8>(), rng.random::<u8>(), rng.random::<u8>())
        }
    }

    pub struct UniformGray;

    impl RandomizationStrategy for UniformGray {
        fn generate_with(&mut self, rng: &mut dyn RngCore) -> Color {
            Color::graytone(rng.random::<f64>())
        }
    }

    pub struct UniformHueLCh;

    impl RandomizationStrategy for UniformHueLCh {
        fn generate_with(&mut self, rng: &mut dyn RngCore) -> Color {
            Color::from_lch(70.0, 35.0, 360.0 * rng.random::<f64>(), 1.0)
        }
    }
}
