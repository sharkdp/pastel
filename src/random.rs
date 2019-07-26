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

    impl RandomizationStrategy for UniformRGB {
        fn generate(&mut self) -> Color {
            Color::from_rgb(random::<u8>(), random::<u8>(), random::<u8>())
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
            Color::from_lch(70.0, 35.0, 360.0 * random::<f64>())
        }
    }
}
