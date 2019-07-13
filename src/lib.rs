mod helper;
mod types;

use helper::{clamp, mod_positive};
use types::Scalar;

/// The hue of a color, represented by an angle (degrees).
#[derive(Debug, Clone, Copy)]
struct Hue {
    unclipped: Scalar,
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

/// The representation of a color.
///
/// Note:
/// - Colors outside the sRGB gamut (which cannot be displayed on a typical
///   computer screen) can not be represented by `Color`.
/// - The `PartialEq` instance compares two `Color`s by comparing their (integer)
///   RGB values. This is different from comparing the HSL values. For example,
///   HSL has many different representations of black (arbitrary hue and
///   saturation values).
#[derive(Debug, Clone)]
pub struct Color {
    hue: Hue,
    saturation: Scalar,
    lightness: Scalar,
    alpha: Scalar,
}

// Illuminant D65 constants used for Lab color space conversions.
const D65_XN: Scalar = 0.950470;
const D65_YN: Scalar = 1.0;
const D65_ZN: Scalar = 1.088830;

impl Color {
    pub fn from_hsla(hue: Scalar, saturation: Scalar, lightness: Scalar, alpha: Scalar) -> Color {
        Color {
            hue: Hue::from(hue),
            saturation: clamp(0.0, 1.0, saturation),
            lightness: clamp(0.0, 1.0, lightness),
            alpha: clamp(0.0, 1.0, alpha),
        }
    }

    ///
    pub fn from_hsl(hue: Scalar, saturation: Scalar, lightness: Scalar) -> Color {
        Self::from_hsla(hue, saturation, lightness, 1.0)
    }

    /// Create a `Color` from integer RGB values between 0 and 255 and a floating
    /// point alpha value between 0.0 and 1.0.
    pub fn from_rgba(r: u8, g: u8, b: u8, alpha: Scalar) -> Color {
        // RGB to HSL conversion algorithm adapted from
        // https://en.wikipedia.org/wiki/HSL_and_HSV

        let max_chroma = u8::max(u8::max(r, g), b);
        let min_chroma = u8::min(u8::min(r, g), b);

        let chroma = max_chroma - min_chroma;
        let chroma_s = Scalar::from(chroma) / 255.0;

        let r_s = Scalar::from(r) / 255.0;
        let g_s = Scalar::from(g) / 255.0;
        let b_s = Scalar::from(b) / 255.0;

        let hue = 60.0
            * (if chroma == 0 {
                0.0
            } else {
                if r == max_chroma {
                    mod_positive((g_s - b_s) / chroma_s, 6.0)
                } else if g == max_chroma {
                    (b_s - r_s) / chroma_s + 2.0
                } else {
                    (r_s - g_s) / chroma_s + 4.0
                }
            });

        let lightness = (Scalar::from(max_chroma) + Scalar::from(min_chroma)) / (255.0 * 2.0);
        let saturation = if chroma == 0 {
            0.0
        } else {
            chroma_s / (1.0 - Scalar::abs(2.0 * lightness - 1.0))
        };

        Self::from_hsla(hue, saturation, lightness, alpha)
    }

    /// Create a `Color` from integer RGB values between 0 and 255.
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Color {
        Self::from_rgba(r, g, b, 1.0)
    }

    /// Create a `Color` from RGB and alpha values between 0.0 and 1.0. Values outside this range
    /// will be clamped.
    pub fn from_rgba_scaled(r: Scalar, g: Scalar, b: Scalar, alpha: Scalar) -> Color {
        let r = Scalar::round(clamp(0.0, 255.0, 255.0 * r)) as u8;
        let g = Scalar::round(clamp(0.0, 255.0, 255.0 * g)) as u8;
        let b = Scalar::round(clamp(0.0, 255.0, 255.0 * b)) as u8;

        Self::from_rgba(r, g, b, alpha)
    }

    /// Create a `Color` from RGB values between 0.0 and 1.0. Values outside this range will be
    /// clamped.
    pub fn from_rgb_scaled(r: Scalar, g: Scalar, b: Scalar) -> Color {
        Self::from_rgba_scaled(r, g, b, 1.0)
    }

    /// Create a `Color` from XYZ coordinates in the CIE 1931 color space. Note that a `Color`
    /// always represents a color in the sRGB gamut (colors that can be represented on a typical
    /// computer screen) while the XYZ color space is bigger. This function will tend to create
    /// fully saturated colors at the edge of the sRGB gamut if the coordinates lie outside the
    /// sRGB range.
    ///
    /// See:
    /// - https://en.wikipedia.org/wiki/CIE_1931_color_space
    /// - https://en.wikipedia.org/wiki/SRGB
    pub fn from_xyz(x: Scalar, y: Scalar, z: Scalar) -> Color {
        let f = |c| {
            if c <= 0.0031308 {
                12.92 * c
            } else {
                1.055 * Scalar::powf(c, 1.0 / 2.4) - 0.055
            }
        };

        let r = f(3.2406 * x - 1.5372 * y - 0.4986 * z);
        let g = f(-0.9689 * x + 1.8758 * y + 0.0415 * z);
        let b = f(0.0557 * x - 0.2040 * y + 1.0570 * z);

        Self::from_rgb_scaled(r, g, b)
    }

    /// Create a `Color` from L, a and b coordinates coordinates in the Lab color
    /// space. Note: See documentation for `from_xyz`. The same restrictions apply here.
    ///
    /// See: https://en.wikipedia.org/wiki/Lab_color_space
    pub fn from_lab(l: Scalar, a: Scalar, b: Scalar) -> Color {
        const DELTA: Scalar = 6.0 / 29.0;

        let finv = |t| {
            if t > DELTA {
                Scalar::powf(t, 3.0)
            } else {
                3.0 * DELTA * DELTA * (t - 4.0 / 29.0)
            }
        };

        let l_ = (l + 16.0) / 116.0;
        let x = D65_XN * finv(l_ + a / 500.0);
        let y = D65_YN * finv(l_);
        let z = D65_ZN * finv(l_ - b / 200.0);

        Self::from_xyz(x, y, z)
    }

    /// Create a `Color` from lightness, chroma and hue coordinates in the CIE LCh color space.
    /// This is a cylindrical transform of the Lab color space. Note: See documentation for
    /// `from_xyz`. The same restrictions apply here.
    ///
    /// See: https://en.wikipedia.org/wiki/Lab_color_space
    pub fn from_lch(l: Scalar, c: Scalar, h: Scalar) -> Color {
        const DEG2RAD: Scalar = std::f64::consts::PI / 180.0;

        let a = c * Scalar::cos(h * DEG2RAD);
        let b = c * Scalar::sin(h * DEG2RAD);

        Self::from_lab(l, a, b)
    }

    /// Convert a `Color` to its hue, saturation, lightness and alpha values. The hue is given
    /// in degrees, as a number between 0.0 and 360.0. Saturation, lightness and alpha are numbers
    /// between 0.0 and 1.0.
    pub fn to_hsla(&self) -> HSLA {
        HSLA {
            h: self.hue.value(),
            s: self.saturation,
            l: self.lightness,
            alpha: self.alpha,
        }
    }

    /// Format the color as a HSL-representation string (`hsl(123, 50%, 80%)`).
    pub fn to_hsl_string(&self) -> String {
        format!(
            "hsl({:.0}, {:.0}%, {:.0}%)",
            self.hue.value(),
            100.0 * self.saturation,
            100.0 * self.lightness
        )
    }

    /// Convert a `Color` to its red, green, blue and alpha values. The RGB values are integers in
    /// the range from 0 to 255. The alpha channel is a number between 0.0 and 1.0.
    pub fn to_rgba(&self) -> RGBA<u8> {
        let c = self.to_rgba_scaled();
        let r = Scalar::round(255.0 * c.r) as u8;
        let g = Scalar::round(255.0 * c.g) as u8;
        let b = Scalar::round(255.0 * c.b) as u8;

        RGBA {
            r,
            g,
            b,
            alpha: self.alpha,
        }
    }

    /// Format the color as a RGB-representation string (`rgb(255, 127,  0)`).
    pub fn to_rgb_string(&self) -> String {
        let rgba = self.to_rgba();
        format!("rgb({}, {}, {})", rgba.r, rgba.g, rgba.b)
    }

    /// Format the color as a RGB-representation string (`#fc0070`).
    pub fn to_rgb_hex_string(&self) -> String {
        let rgba = self.to_rgba();
        format!("#{:02x}{:02x}{:02x}", rgba.r, rgba.g, rgba.b)
    }

    /// Convert a `Color` to its red, green, blue and alpha values. All numbers are from the range
    /// between 0.0 and 1.0.
    pub fn to_rgba_scaled(&self) -> RGBA<Scalar> {
        let h_s = self.hue.value() / 60.0;
        let chr = (1.0 - Scalar::abs(2.0 * self.lightness - 1.0)) * self.saturation;
        let m = self.lightness - chr / 2.0;
        let x = chr * (1.0 - Scalar::abs(h_s % 2.0 - 1.0));

        struct RGB(Scalar, Scalar, Scalar);

        let col = if h_s < 1.0 {
            RGB(chr, x, 0.0)
        } else if 1.0 <= h_s && h_s < 2.0 {
            RGB(x, chr, 0.0)
        } else if 2.0 <= h_s && h_s < 3.0 {
            RGB(0.0, chr, x)
        } else if 3.0 <= h_s && h_s < 4.0 {
            RGB(0.0, x, chr)
        } else if 4.0 <= h_s && h_s < 5.0 {
            RGB(x, 0.0, chr)
        } else {
            RGB(chr, 0.0, x)
        };

        RGBA {
            r: col.0 + m,
            g: col.1 + m,
            b: col.2 + m,
            alpha: self.alpha,
        }
    }

    /// Get XYZ coordinates according to the CIE 1931 color space.
    ///
    /// See:
    /// - https://en.wikipedia.org/wiki/CIE_1931_color_space
    /// - https://en.wikipedia.org/wiki/SRGB
    pub fn to_xyz(&self) -> XYZ {
        let finv = |c_| {
            if c_ <= 0.04045 {
                c_ / 12.92
            } else {
                Scalar::powf((c_ + 0.055) / 1.055, 2.4)
            }
        };

        let rec = self.to_rgba_scaled();
        let r = finv(rec.r);
        let g = finv(rec.g);
        let b = finv(rec.b);

        let x = 0.4124 * r + 0.3576 * g + 0.1805 * b;
        let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let z = 0.0193 * r + 0.1192 * g + 0.9505 * b;

        XYZ {
            x,
            y,
            z,
            alpha: self.alpha,
        }
    }

    /// Get L, a and b coordinates according to the Lab color space.
    ///
    /// See: https://en.wikipedia.org/wiki/Lab_color_space
    pub fn to_lab(&self) -> Lab {
        let rec = self.to_xyz();

        let cut = Scalar::powf(6.0 / 29.0, 3.0);
        let f = |t| {
            if t > cut {
                Scalar::powf(t, 1.0 / 3.0)
            } else {
                (1.0 / 3.0) * Scalar::powf(29.0 / 6.0, 2.0) * t + 4.0 / 29.0
            }
        };

        let fy = f(rec.y / D65_YN);

        let l = 116.0 * fy - 16.0;
        let a = 500.0 * (f(rec.x / D65_XN) - fy);
        let b = 200.0 * (fy - f(rec.z / D65_ZN));

        Lab {
            l,
            a,
            b,
            alpha: self.alpha,
        }
    }

    /// Get L, C and h coordinates according to the CIE LCh color space.
    ///
    /// See: https://en.wikipedia.org/wiki/Lab_color_space
    pub fn to_lch(&self) -> LCh {
        let Lab { l, a, b, alpha } = self.to_lab();

        const RAD2DEG: Scalar = 180.0 / std::f64::consts::PI;

        let c = Scalar::sqrt(a * a + b * b);
        let h = mod_positive(Scalar::atan2(b, a) * RAD2DEG, 360.0);

        LCh { l, c, h, alpha }
    }

    /// Pure black.
    pub fn black() -> Color {
        Color::from_hsl(0.0, 0.0, 0.0)
    }

    /// Pure white.
    pub fn white() -> Color {
        Color::from_hsl(0.0, 0.0, 1.0)
    }

    /// Create a gray tone from a lightness value (0.0 is black, 1.0 is white).
    pub fn graytone(lightness: Scalar) -> Color {
        Color::from_hsl(0.0, 0.0, lightness)
    }

    /// Rotate along the "hue" axis.
    pub fn rotate_hue(&self, delta: Scalar) -> Color {
        Self::from_hsla(
            self.hue.value() + delta,
            self.saturation,
            self.lightness,
            self.alpha,
        )
    }

    /// Get the complementary color (hue rotated by 180°).
    pub fn complementary(&self) -> Color {
        self.rotate_hue(180.0)
    }

    /// Lighten a color by adding a certain amount (number between -1.0 and 1.0) to the lightness
    /// channel. If the number is negative, the color is darkened.
    pub fn lighten(&self, f: Scalar) -> Color {
        Self::from_hsla(
            self.hue.value(),
            self.saturation,
            self.lightness + f,
            self.alpha,
        )
    }

    /// Darken a color by subtracting a certain amount (number between -1.0 and 1.0) from the
    /// lightness channel. If the number is negative, the color is lightened.
    pub fn darken(&self, f: Scalar) -> Color {
        self.lighten(-f)
    }

    /// Increase the saturation of a color by adding a certain amount (number between -1.0 and 1.0)
    /// to the saturation channel. If the number is negative, the color is desaturated.
    pub fn saturate(&self, f: Scalar) -> Color {
        Self::from_hsla(
            self.hue.value(),
            self.saturation + f,
            self.lightness,
            self.alpha,
        )
    }

    /// Decrease the saturation of a color by subtracting a certain amount (number between -1.0 and
    /// 1.0) from the saturation channel. If the number is negative, the color is saturated.
    pub fn desaturate(&self, f: Scalar) -> Color {
        self.saturate(-f)
    }

    /// Convert a color to a gray tone with the same perceived luminance (see `luminance`).
    pub fn to_gray(&self) -> Color {
        let hue = self.hue;
        let c = self.to_lch();

        // the desaturation step is only needed to correct minor rounding errors.
        let mut gray = Color::from_lch(c.l, 0.0, 0.0).desaturate(1.0);

        // Restore the hue value (does not alter the color, but makes it able to add saturation
        // again)
        gray.hue = hue;

        gray
    }

    /// The percieved brightness of the color (A number between 0.0 and 1.0).
    ///
    /// See: https://www.w3.org/TR/AERT#color-contrast
    pub fn brightness(&self) -> Scalar {
        let c = self.to_rgba_scaled();
        (299.0 * c.r + 587.0 * c.g + 114.0 * c.b) / 1000.0
    }

    /// Determine whether a color is perceived as a light color (perceived brightness is larger
    /// than 0.5).
    pub fn is_light(&self) -> bool {
        self.brightness() > 0.5
    }

    /// The relative brightness of a color (normalized to 0.0 for darkest black
    /// and 1.0 for lightest white), according to the WCAG definition.
    ///
    /// See: https://www.w3.org/TR/2008/REC-WCAG20-20081211/#relativeluminancedef
    pub fn luminance(&self) -> Scalar {
        fn f(s: Scalar) -> Scalar {
            if s <= 0.03928 {
                s / 12.92
            } else {
                Scalar::powf((s + 0.055) / 1.055, 2.4)
            }
        };
        let c = self.to_rgba_scaled();
        let r = f(c.r);
        let g = f(c.g);
        let b = f(c.b);

        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// Return a readable foreground text color (either `black` or `white`) for a
    /// given background color.
    pub fn text_color(&self) -> Color {
        if self.is_light() {
            Self::black()
        } else {
            Self::white()
        }
    }

    /// Compute the perceived 'distance' between two colors according to the CIE76 delta-E
    /// standard. A distance below ~2.3 is not noticable.
    ///
    /// See: https://en.wikipedia.org/wiki/Color_difference
    pub fn distance(&self, other: &Color) -> Scalar {
        let c1 = self.to_lab();
        let c2 = other.to_lab();

        ((c1.l - c2.l).powi(2) + (c1.a - c2.a).powi(2) + (c1.b - c2.b).powi(2)).sqrt()
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Color) -> bool {
        self.to_rgba() == other.to_rgba()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RGBA<T> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub alpha: Scalar,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HSLA {
    pub h: Scalar,
    pub s: Scalar,
    pub l: Scalar,
    pub alpha: Scalar,
}

#[derive(Debug, Clone, PartialEq)]
pub struct XYZ {
    pub x: Scalar,
    pub y: Scalar,
    pub z: Scalar,
    pub alpha: Scalar,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lab {
    pub l: Scalar,
    pub a: Scalar,
    pub b: Scalar,
    pub alpha: Scalar,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LCh {
    pub l: Scalar,
    pub c: Scalar,
    pub h: Scalar,
    pub alpha: Scalar,
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn assert_almost_equal(c1: &Color, c2: &Color) {
        let c1 = c1.to_rgba();
        let c2 = c2.to_rgba();

        assert!((c1.r as i32 - c2.r as i32).abs() <= 1);
        assert!((c1.g as i32 - c2.g as i32).abs() <= 1);
        assert!((c1.b as i32 - c2.b as i32).abs() <= 1);
    }

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

    #[test]
    fn test_color_partial_eq() {
        assert_eq!(
            Color::from_hsl(120.0, 0.3, 0.5),
            Color::from_hsl(360.0 + 120.0, 0.3, 0.5),
        );
        assert_eq!(
            Color::from_rgba(1, 2, 3, 0.3),
            Color::from_rgba(1, 2, 3, 0.3),
        );
        assert_eq!(Color::black(), Color::from_hsl(123.0, 0.3, 0.0));
        assert_eq!(Color::white(), Color::from_hsl(123.0, 0.3, 1.0));

        assert_ne!(
            Color::from_hsl(120.0, 0.3, 0.5),
            Color::from_hsl(122.0, 0.3, 0.5),
        );
        assert_ne!(
            Color::from_hsl(120.0, 0.3, 0.5),
            Color::from_hsl(120.0, 0.32, 0.5),
        );
        assert_ne!(
            Color::from_hsl(120.0, 0.3, 0.5),
            Color::from_hsl(120.0, 0.3, 0.52),
        );
        assert_ne!(
            Color::from_hsla(120.0, 0.3, 0.5, 0.9),
            Color::from_hsla(120.0, 0.3, 0.5, 0.901),
        );
        assert_ne!(
            Color::from_rgba(1, 2, 3, 0.3),
            Color::from_rgba(2, 2, 3, 0.3),
        );
        assert_ne!(
            Color::from_rgba(1, 2, 3, 0.3),
            Color::from_rgba(1, 3, 3, 0.3),
        );
        assert_ne!(
            Color::from_rgba(1, 2, 3, 0.3),
            Color::from_rgba(1, 2, 4, 0.3),
        );
    }

    #[test]
    fn test_rgb_to_hsl_conversion() {
        assert_eq!(
            Color::from_hsl(0.0, 0.0, 1.0),
            Color::from_rgb_scaled(1.0, 1.0, 1.0)
        ); // white
        assert_eq!(
            Color::from_hsl(0.0, 0.0, 0.5),
            Color::from_rgb_scaled(0.5, 0.5, 0.5)
        ); // gray
        assert_eq!(
            Color::from_hsl(0.0, 0.0, 0.0),
            Color::from_rgb_scaled(0.0, 0.0, 0.0)
        ); // black
        assert_eq!(
            Color::from_hsl(0.0, 1.0, 0.5),
            Color::from_rgb_scaled(1.0, 0.0, 0.0)
        ); // red
        assert_eq!(
            Color::from_hsl(60.0, 1.0, 0.375),
            Color::from_rgb_scaled(0.75, 0.75, 0.0)
        ); //yellow-green
        assert_eq!(
            Color::from_hsl(120.0, 1.0, 0.25),
            Color::from_rgb_scaled(0.0, 0.5, 0.0)
        ); // green
        assert_eq!(
            Color::from_hsl(240.0, 1.0, 0.75),
            Color::from_rgb_scaled(0.5, 0.5, 1.0)
        ); // blue
        assert_eq!(
            Color::from_hsl(49.5, 0.893, 0.497),
            Color::from_rgb_scaled(0.941, 0.785, 0.053)
        ); // yellow
        assert_eq!(
            Color::from_hsl(162.4, 0.779, 0.447),
            Color::from_rgb_scaled(0.099, 0.795, 0.591)
        ); // cyan 2
    }

    #[test]
    fn test_rgb_roundtrip_conversion() {
        let roundtrip = |h, s, l| {
            let color1 = Color::from_hsl(h, s, l);
            let rgb = color1.to_rgba();
            let color2 = Color::from_rgb(rgb.r, rgb.g, rgb.b);
            assert_eq!(color1, color2);
        };

        roundtrip(0.0, 0.0, 1.0);
        roundtrip(0.0, 0.0, 0.5);
        roundtrip(0.0, 0.0, 0.0);
        roundtrip(60.0, 1.0, 0.375);
        roundtrip(120.0, 1.0, 0.25);
        roundtrip(240.0, 1.0, 0.75);
        roundtrip(49.5, 0.893, 0.497);
        roundtrip(162.4, 0.779, 0.447);

        for degree in 0..360 {
            roundtrip(Scalar::from(degree), 0.5, 0.8);
        }
    }

    #[test]
    fn test_xyz_conversion() {
        assert_eq!(Color::white(), Color::from_xyz(0.9505, 1.0, 1.0890));
        assert_eq!(
            Color::from_rgb(255, 0, 0),
            Color::from_xyz(0.4123, 0.2126, 0.01933)
        );
        assert_eq!(
            Color::from_hsl(109.999, 0.08654, 0.407843),
            Color::from_xyz(0.13123, 0.15372, 0.13174)
        );

        let roundtrip = |h, s, l| {
            let color1 = Color::from_hsl(h, s, l);
            let xyz1 = color1.to_xyz();
            let color2 = Color::from_xyz(xyz1.x, xyz1.y, xyz1.z);
            assert_almost_equal(&color1, &color2);
        };

        for hue in 0..360 {
            roundtrip(Scalar::from(hue), 0.2, 0.8);
        }
    }

    #[test]
    fn test_lab_conversion() {
        assert_eq!(
            Color::from_rgb(255, 0, 0),
            Color::from_lab(53.233, 80.109, 67.22)
        );

        let roundtrip = |h, s, l| {
            let color1 = Color::from_hsl(h, s, l);
            let lab1 = color1.to_lab();
            let color2 = Color::from_lab(lab1.l, lab1.a, lab1.b);
            assert_almost_equal(&color1, &color2);
        };

        for hue in 0..360 {
            roundtrip(Scalar::from(hue), 0.2, 0.8);
        }
    }

    #[test]
    fn test_lch_conversion() {
        assert_eq!(
            Color::from_hsl(0.0, 1.0, 0.245),
            Color::from_lch(24.829, 60.093, 38.18)
        );

        let roundtrip = |h, s, l| {
            let color1 = Color::from_hsl(h, s, l);
            let lch1 = color1.to_lch();
            let color2 = Color::from_lch(lch1.l, lch1.c, lch1.h);
            assert_almost_equal(&color1, &color2);
        };

        for hue in 0..360 {
            roundtrip(Scalar::from(hue), 0.2, 0.8);
        }
    }

    #[test]
    fn test_rotate_hue() {
        assert_eq!(
            Color::from_rgb(0, 255, 0),
            Color::from_rgb(255, 0, 0).rotate_hue(120.0)
        );
    }

    #[test]
    fn test_complementary() {
        let magenta = Color::from_rgb(255, 0, 255);
        let lime = Color::from_rgb(0, 255, 0);
        assert_eq!(magenta, lime.complementary());

        let magenta = Color::from_rgb(255, 0, 255);
        let lime = Color::from_rgb(0, 255, 0);
        assert_eq!(magenta, lime.complementary());
    }

    #[test]
    fn test_lighten() {
        assert_eq!(
            Color::from_hsl(90.0, 0.5, 0.7),
            Color::from_hsl(90.0, 0.5, 0.3).lighten(0.4)
        );
        assert_eq!(
            Color::from_hsl(90.0, 0.5, 1.0),
            Color::from_hsl(90.0, 0.5, 0.3).lighten(0.8)
        );
    }

    #[test]
    fn test_to_gray() {
        let salmon = Color::from_rgb(250, 128, 114);
        assert_eq!(0.0, salmon.to_gray().to_hsla().s);
        assert_relative_eq!(
            salmon.luminance(),
            salmon.to_gray().luminance(),
            max_relative = 0.01
        );

        assert_eq!(Color::graytone(0.3), Color::graytone(0.3).to_gray());
    }

    #[test]
    fn test_brightness() {
        assert_eq!(0.0, Color::black().brightness());
        assert_eq!(1.0, Color::white().brightness());
        assert_eq!(0.5, Color::graytone(0.5).brightness());
    }

    #[test]
    fn test_luminance() {
        assert_eq!(1.0, Color::white().luminance());
        let hotpink = Color::from_rgb(255, 105, 180);
        assert_relative_eq!(0.347, hotpink.luminance(), max_relative = 0.01);
        assert_eq!(0.0, Color::black().luminance());
    }

    #[test]
    fn test_text_color() {
        assert_eq!(Color::white(), Color::graytone(0.4).text_color());
        assert_eq!(Color::black(), Color::graytone(0.6).text_color());
    }

    #[test]
    fn test_distance() {
        let c = Color::from_rgb(255, 127, 14);
        assert_eq!(0.0, c.distance(&c));

        let c1 = Color::from_rgb(50, 100, 200);
        let c2 = Color::from_rgb(200, 10, 0);
        assert_eq!(123.0, c1.distance(&c2).round());
    }

    #[test]
    fn test_to_hsl_string() {
        let c = Color::from_hsl(91.3, 0.54, 0.98);
        assert_eq!("hsl(91, 54%, 98%)", c.to_hsl_string());
    }

    #[test]
    fn test_to_rgb_string() {
        let c = Color::from_rgb(255, 127, 4);
        assert_eq!("rgb(255, 127, 4)", c.to_rgb_string());
    }

    #[test]
    fn test_to_rgb_hex_string() {
        let c = Color::from_rgb(255, 127, 4);
        assert_eq!("#ff7f04", c.to_rgb_hex_string());
    }
}
