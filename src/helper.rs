use std::cmp::Ordering;

use crate::types::Scalar;

/// Like `%`, but always positive.
pub fn mod_positive(x: Scalar, y: Scalar) -> Scalar {
    (x % y + y) % y
}

/// Trim a number such that it fits into the range [lower, upper].
pub fn clamp(lower: Scalar, upper: Scalar, x: Scalar) -> Scalar {
    Scalar::max(Scalar::min(upper, x), lower)
}

#[derive(Debug, Clone, Copy)]
pub struct Fraction {
    f: Scalar,
}

impl Fraction {
    pub fn from(s: Scalar) -> Self {
        Fraction {
            f: clamp(0.0, 1.0, s),
        }
    }

    pub fn value(self) -> Scalar {
        self.f
    }
}

/// Linearly interpolate between two values.
pub fn interpolate(a: Scalar, b: Scalar, fraction: Fraction) -> Scalar {
    a + fraction.value() * (b - a)
}

/// Linearly interpolate between two angles. Always take the shortest path
/// along the circle.
pub fn interpolate_angle(a: Scalar, b: Scalar, fraction: Fraction) -> Scalar {
    let paths = [(a, b), (a, b + 360.0), (a + 360.0, b)];

    let dist = |&(x, y): &(Scalar, Scalar)| (x - y).abs();
    let shortest = paths
        .iter()
        .min_by(|p1, p2| dist(p1).partial_cmp(&dist(p2)).unwrap_or(Ordering::Less))
        .unwrap();

    mod_positive(interpolate(shortest.0, shortest.1, fraction), 360.0)
}

#[test]
fn test_interpolate() {
    assert_eq!(0.0, interpolate_angle(0.0, 90.0, Fraction::from(0.0)));
    assert_eq!(45.0, interpolate_angle(0.0, 90.0, Fraction::from(0.5)));
    assert_eq!(90.0, interpolate_angle(0.0, 90.0, Fraction::from(1.0)));
    assert_eq!(90.0, interpolate_angle(0.0, 90.0, Fraction::from(1.1)));
}

#[test]
fn test_interpolate_angle() {
    assert_eq!(15.0, interpolate_angle(0.0, 30.0, Fraction::from(0.5)));
    assert_eq!(20.0, interpolate_angle(0.0, 100.0, Fraction::from(0.2)));
    assert_eq!(0.0, interpolate_angle(10.0, 350.0, Fraction::from(0.5)));
    assert_eq!(0.0, interpolate_angle(350.0, 10.0, Fraction::from(0.5)));
}
