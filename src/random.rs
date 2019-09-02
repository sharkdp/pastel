use crate::Color;

pub trait RandomizationStrategy {
    fn generate(&mut self) -> Color;
}

pub mod strategies {
    use super::RandomizationStrategy;
    use crate::Color;

    use rand::prelude::*;

    pub struct Vivid;

    impl RandomizationStrategy for Vivid {
        fn generate(&mut self) -> Color {
            let hue = random::<f64>() * 360.0;
            let saturation = 0.2 + 0.6 * random::<f64>();
            let lightness = 0.3 + 0.4 * random::<f64>();

            Color::from_hsl(hue, saturation, lightness)
        }
    }

    pub struct UniformRGB;

    impl UniformRGB {
        pub fn generate_with(&self, rng: &mut impl Rng) -> Color {
            Color::from_rgb(rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>())
        }
    }

    impl RandomizationStrategy for UniformRGB {
        fn generate(&mut self) -> Color {
            self.generate_with(&mut thread_rng())
        }
    }

    pub struct UniformGray;

    impl RandomizationStrategy for UniformGray {
        fn generate(&mut self) -> Color {
            Color::graytone(random::<f64>())
        }
    }

    pub struct UniformHueLCh;

    impl RandomizationStrategy for UniformHueLCh {
        fn generate(&mut self) -> Color {
            Color::from_lch(70.0, 35.0, 360.0 * random::<f64>(), 1.0)
        }
    }
}
