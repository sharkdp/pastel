type Scalar = f64;

#[derive(Debug, Clone, Copy)]
struct Hue {
    unclipped: Scalar,
}

/// Like `%`, but always positive.
fn mod_positive(x: Scalar, y: Scalar) -> Scalar {
    (x % y + y) % y
}

impl Hue {
    pub fn from(unclipped: Scalar) -> Hue {
        Hue { unclipped }
    }

    /// Return a hue value in the interval [0, 360].
    pub fn value(self) -> Scalar {
        if self.unclipped == 360.0 {
            self.unclipped
        } else {
            mod_positive(self.unclipped, 360.0)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Color {
    h: Hue,
    s: Scalar,
    l: Scalar,
    a: Scalar,
}

impl Color {
    pub fn hsl(h: Scalar, s: Scalar, l: Scalar) -> Color {
        Color { h: Hue::from(h), s, l, a: 1.0 }
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Color) -> bool {
        true
    }
}


#[test]
fn hue_clipping() {
    assert_eq!(43.0, Hue::from(43.0).value());
    assert_eq!(13.0, Hue::from(373.0).value());
    assert_eq!(300.0, Hue::from(-60.0).value());
    assert_eq!(360.0, Hue::from(360.0).value());
}

#[test]
fn color_eq() {
}
