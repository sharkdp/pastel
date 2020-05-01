use crate::helper::mod_positive;

pub type Scalar = f64;

/// The hue of a color, represented by an angle (degrees).
#[derive(Debug, Clone, Copy)]
pub struct Hue {
    unclipped: Scalar,
}

impl Hue {
    pub fn from(unclipped: Scalar) -> Hue {
        Hue { unclipped }
    }

    /// Return a hue value in the interval [0, 360].
    pub fn value(self) -> Scalar {
        #![allow(clippy::float_cmp)]
        if self.unclipped == 360.0 {
            self.unclipped
        } else {
            mod_positive(self.unclipped, 360.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_mod_positive() {
        assert_relative_eq!(0.5, mod_positive(2.9, 2.4));
        assert_relative_eq!(1.7, mod_positive(-0.3, 2.0));
    }

    #[test]
    fn test_hue_clipping() {
        assert_eq!(43.0, Hue::from(43.0).value());
        assert_eq!(13.0, Hue::from(373.0).value());
        assert_eq!(300.0, Hue::from(-60.0).value());
        assert_eq!(360.0, Hue::from(360.0).value());
    }
}
