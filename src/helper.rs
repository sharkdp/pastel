use std::{
    cmp::Ordering,
    fmt::{self, Display},
};

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

// `format!`-style format strings only allow specifying a fixed floating
// point precision, e.g. `{:.3}` to print 3 decimal places. This always
// displays trailing zeroes, while web colors generally omit them. For
// example, we'd prefer to print `0.5` as `0.5` instead of `0.500`.
//
// Note that this will round using omitted decimal places:
//
//     MaxPrecision::wrap(3, 0.5004) //=> 0.500
//     MaxPrecision::wrap(3, 0.5005) //=> 0.501
//
pub struct MaxPrecision {
    precision: u32,
    inner: f64,
}

impl MaxPrecision {
    pub fn wrap(precision: u32, inner: f64) -> Self {
        Self { precision, inner }
    }
}

impl Display for MaxPrecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pow_10 = 10u32.pow(self.precision) as f64;
        let rounded = (self.inner * pow_10).round() / pow_10;
        write!(f, "{}", rounded)
    }
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

#[test]
fn test_max_precision() {
    assert_eq!(format!("{}", MaxPrecision::wrap(3, 0.5)), "0.5");
    assert_eq!(format!("{}", MaxPrecision::wrap(3, 0.51)), "0.51");
    assert_eq!(format!("{}", MaxPrecision::wrap(3, 0.512)), "0.512");
    assert_eq!(format!("{}", MaxPrecision::wrap(3, 0.5124)), "0.512");
    assert_eq!(format!("{}", MaxPrecision::wrap(3, 0.5125)), "0.513");
}
