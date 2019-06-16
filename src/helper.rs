use crate::types::Scalar;

/// Like `%`, but always positive.
pub fn mod_positive(x: Scalar, y: Scalar) -> Scalar {
    (x % y + y) % y
}

/// Trim a number such that it fits into the range [lower, upper].
pub fn clamp(lower: Scalar, upper: Scalar, x: Scalar) -> Scalar {
    Scalar::max(Scalar::min(upper, x), lower)
}
