use crate::helper::Fraction;
use crate::Color;

pub trait ColorSpace {
    fn from_color(c: &Color) -> Self;
    // NOTE: Clippy's suggestion might be correct here
    #[allow(clippy::wrong_self_convention)]
    fn into_color(&self) -> Color;

    fn mix(&self, other: &Self, fraction: Fraction) -> Self;
}
