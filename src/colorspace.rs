use crate::helper::Fraction;
use crate::Color;

pub trait ColorSpace {
    fn from_color(c: &Color) -> Self;
    fn into_color(self) -> Color;

    fn mix(&self, other: &Self, fraction: Fraction) -> Self;
}
