pub mod ansi;
pub mod colorspace;
pub mod delta_e;
pub mod distinct;
mod helper;
pub mod named;
pub mod parser;
pub mod random;
mod types;

use std::fmt;

use colorspace::ColorSpace;
pub use helper::Fraction;
use helper::{clamp, interpolate, interpolate_angle, mod_positive};
use types::{Hue, Scalar};

/// The representation of a color.
///
/// Note:
/// - Colors outside the sRGB gamut (which cannot be displayed on a typical
///   computer screen) can not be represented by `Color`.
/// - The `PartialEq` instance compares two `Color`s by comparing their (integer)
///   RGB values. This is different from comparing the HSL values. For example,
///   HSL has many different representations of black (arbitrary hue and
///   saturation values).
#[derive(Clone)]
pub struct Color {
    hue: Hue,
    saturation: Scalar,
    lightness: Scalar,
    alpha: Scalar,
}

// Illuminant D65 constants used for Lab color space conversions.
const D65_XN: Scalar = 0.950_470;
const D65_YN: Scalar = 1.0;
const D65_ZN: Scalar = 1.088_830;

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
            } else if r == max_chroma {
                mod_positive((g_s - b_s) / chroma_s, 6.0)
            } else if g == max_chroma {
                (b_s - r_s) / chroma_s + 2.0
            } else {
                (r_s - g_s) / chroma_s + 4.0
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
    pub fn from_rgba_float(r: Scalar, g: Scalar, b: Scalar, alpha: Scalar) -> Color {
        let r = Scalar::round(clamp(0.0, 255.0, 255.0 * r)) as u8;
        let g = Scalar::round(clamp(0.0, 255.0, 255.0 * g)) as u8;
        let b = Scalar::round(clamp(0.0, 255.0, 255.0 * b)) as u8;

        Self::from_rgba(r, g, b, alpha)
    }

    /// Create a `Color` from RGB values between 0.0 and 1.0. Values outside this range will be
    /// clamped.
    pub fn from_rgb_float(r: Scalar, g: Scalar, b: Scalar) -> Color {
        Self::from_rgba_float(r, g, b, 1.0)
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
    pub fn from_xyz(x: Scalar, y: Scalar, z: Scalar, alpha: Scalar) -> Color {
        #![allow(clippy::many_single_char_names)]
        let f = |c| {
            if c <= 0.003_130_8 {
                12.92 * c
            } else {
                1.055 * Scalar::powf(c, 1.0 / 2.4) - 0.055
            }
        };

        let r = f(3.2406 * x - 1.5372 * y - 0.4986 * z);
        let g = f(-0.9689 * x + 1.8758 * y + 0.0415 * z);
        let b = f(0.0557 * x - 0.2040 * y + 1.0570 * z);

        Self::from_rgba_float(r, g, b, alpha)
    }

    /// Create a `Color` from LMS coordinates. This is the matrix inverse of the matrix that
    /// appears in `to_lms`.
    pub fn from_lms(l: Scalar, m: Scalar, s: Scalar, alpha: Scalar) -> Color {
        #![allow(clippy::many_single_char_names)]
        let x = 1.91020 * l - 1.112_120 * m + 0.201_908 * s;
        let y = 0.37095 * l + 0.629_054 * m + 0.000_000 * s;
        let z = 0.00000 * l + 0.000_000 * m + 1.000_000 * s;
        Color::from_xyz(x, y, z, alpha)
    }

    /// Create a `Color` from L, a and b coordinates coordinates in the Lab color
    /// space. Note: See documentation for `from_xyz`. The same restrictions apply here.
    ///
    /// See: https://en.wikipedia.org/wiki/Lab_color_space
    pub fn from_lab(l: Scalar, a: Scalar, b: Scalar, alpha: Scalar) -> Color {
        #![allow(clippy::many_single_char_names)]
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

        Self::from_xyz(x, y, z, alpha)
    }

    /// Create a `Color` from lightness, chroma and hue coordinates in the CIE LCh color space.
    /// This is a cylindrical transform of the Lab color space. Note: See documentation for
    /// `from_xyz`. The same restrictions apply here.
    ///
    /// See: https://en.wikipedia.org/wiki/Lab_color_space
    pub fn from_lch(l: Scalar, c: Scalar, h: Scalar, alpha: Scalar) -> Color {
        #![allow(clippy::many_single_char_names)]
        const DEG2RAD: Scalar = std::f64::consts::PI / 180.0;

        let a = c * Scalar::cos(h * DEG2RAD);
        let b = c * Scalar::sin(h * DEG2RAD);

        Self::from_lab(l, a, b, alpha)
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

    /// Format the color as a HSL-representation string (`hsl(123, 50.3%, 80.1%)`).
    pub fn to_hsl_string(&self, format: Format) -> String {
        format!(
            "hsl({:.0},{space}{:.1}%,{space}{:.1}%)",
            self.hue.value(),
            100.0 * self.saturation,
            100.0 * self.lightness,
            space = if format == Format::Spaces { " " } else { "" }
        )
    }

    /// Convert a `Color` to its red, green, blue and alpha values. The RGB values are integers in
    /// the range from 0 to 255. The alpha channel is a number between 0.0 and 1.0.
    pub fn to_rgba(&self) -> RGBA<u8> {
        let c = self.to_rgba_float();
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
    pub fn to_rgb_string(&self, format: Format) -> String {
        let rgba = self.to_rgba();
        format!(
            "rgb({r},{space}{g},{space}{b})",
            r = rgba.r,
            g = rgba.g,
            b = rgba.b,
            space = if format == Format::Spaces { " " } else { "" }
        )
    }

    /// Convert a `Color` to its cyan, magenta, yellow, and black values. The CMYK
    /// values are floats smaller than or equal to 1.0.
    pub fn to_cmyk(&self) -> CMYK {
        let rgba = self.to_rgba();
        let r = (rgba.r as f64) / 255.0;
        let g = (rgba.g as f64) / 255.0;
        let b = (rgba.b as f64) / 255.0;
        let biggest = if r >= g && r >= b {
            r
        } else if g >= r && g >= b {
            g
        } else {
            b
        };
        let k = 1.0 - biggest;
        let c = (1.0 - r - k) / biggest;
        let m = (1.0 - g - k) / biggest;
        let y = (1.0 - b - k) / biggest;

        CMYK {
            c: if c.is_nan() { 0.0 } else { c },
            m: if m.is_nan() { 0.0 } else { m },
            y: if y.is_nan() { 0.0 } else { y },
            k: k,
        }
    }

    /// Format the color as a CMYK-representation string (`cmyk(0, 50, 100, 100)`).
    pub fn to_cmyk_string(&self, format: Format) -> String {
        let cmyk = self.to_cmyk();
        format!(
            "cmyk({c},{space}{m},{space}{y},{space}{k})",
            c = (cmyk.c * 100.0).round(),
            m = (cmyk.m * 100.0).round(),
            y = (cmyk.y * 100.0).round(),
            k = (cmyk.k * 100.0).round(),
            space = if format == Format::Spaces { " " } else { "" }
        )
    }

    /// Format the color as a floating point RGB-representation string (`rgb(1.0, 0.5,  0)`).
    pub fn to_rgb_float_string(&self, format: Format) -> String {
        let rgba = self.to_rgba_float();
        format!(
            "rgb({r:.3},{space}{g:.3},{space}{b:.3})",
            r = rgba.r,
            g = rgba.g,
            b = rgba.b,
            space = if format == Format::Spaces { " " } else { "" }
        )
    }

    /// Format the color as a RGB-representation string (`#fc0070`).
    pub fn to_rgb_hex_string(&self, leading_hash: bool) -> String {
        let rgba = self.to_rgba();
        format!(
            "{}{:02x}{:02x}{:02x}",
            if leading_hash { "#" } else { "" },
            rgba.r,
            rgba.g,
            rgba.b
        )
    }

    /// Convert a `Color` to its red, green, blue and alpha values. All numbers are from the range
    /// between 0.0 and 1.0.
    pub fn to_rgba_float(&self) -> RGBA<Scalar> {
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

    /// Return the color as an integer in RGB representation (`0xRRGGBB`)
    pub fn to_u32(&self) -> u32 {
        let rgba = self.to_rgba();
        u32::from(rgba.r).wrapping_shl(16) + u32::from(rgba.g).wrapping_shl(8) + u32::from(rgba.b)
    }

    /// Get XYZ coordinates according to the CIE 1931 color space.
    ///
    /// See:
    /// - https://en.wikipedia.org/wiki/CIE_1931_color_space
    /// - https://en.wikipedia.org/wiki/SRGB
    pub fn to_xyz(&self) -> XYZ {
        #![allow(clippy::many_single_char_names)]
        let finv = |c_| {
            if c_ <= 0.04045 {
                c_ / 12.92
            } else {
                Scalar::powf((c_ + 0.055) / 1.055, 2.4)
            }
        };

        let rec = self.to_rgba_float();
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

    /// Get coordinates according to the LSM color space
    ///
    /// See https://en.wikipedia.org/wiki/LMS_color_space for info on the color space as well as an
    /// algorithm for converting from CIE XYZ
    pub fn to_lms(&self) -> LMS {
        let XYZ { x, y, z, alpha } = self.to_xyz();
        let l = 0.38971 * x + 0.68898 * y - 0.07868 * z;
        let m = -0.22981 * x + 1.18340 * y + 0.04641 * z;
        let s = 0.00000 * x + 0.00000 * y + 1.00000 * z;

        LMS { l, m, s, alpha }
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

    /// Format the color as a Lab-representation string (`Lab(41, 83, -93)`).
    pub fn to_lab_string(&self, format: Format) -> String {
        let lab = self.to_lab();
        format!(
            "Lab({l:.0},{space}{a:.0},{space}{b:.0})",
            l = lab.l,
            a = lab.a,
            b = lab.b,
            space = if format == Format::Spaces { " " } else { "" }
        )
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

    /// Format the color as a LCh-representation string (`LCh(0.3, 0.2, 0.1)`).
    pub fn to_lch_string(&self, format: Format) -> String {
        let lch = self.to_lch();
        format!(
            "LCh({l:.0},{space}{c:.0},{space}{h:.0})",
            l = lch.l,
            c = lch.c,
            h = lch.h,
            space = if format == Format::Spaces { " " } else { "" }
        )
    }

    /// Pure black.
    pub fn black() -> Color {
        Color::from_hsl(0.0, 0.0, 0.0)
    }

    /// Pure white.
    pub fn white() -> Color {
        Color::from_hsl(0.0, 0.0, 1.0)
    }

    /// Red (`#ff0000`)
    pub fn red() -> Color {
        Color::from_rgb(255, 0, 0)
    }

    /// Green (`#008000`)
    pub fn green() -> Color {
        Color::from_rgb(0, 128, 0)
    }

    /// Blue (`#0000ff`)
    pub fn blue() -> Color {
        Color::from_rgb(0, 0, 255)
    }

    /// Yellow (`#ffff00`)
    pub fn yellow() -> Color {
        Color::from_rgb(255, 255, 0)
    }

    /// Fuchsia (`#ff00ff`)
    pub fn fuchsia() -> Color {
        Color::from_rgb(255, 0, 255)
    }

    /// Aqua (`#00ffff`)
    pub fn aqua() -> Color {
        Color::from_rgb(0, 255, 255)
    }

    /// Lime (`#00ff00`)
    pub fn lime() -> Color {
        Color::from_rgb(0, 255, 0)
    }

    /// Maroon (`#800000`)
    pub fn maroon() -> Color {
        Color::from_rgb(128, 0, 0)
    }

    /// Olive (`#808000`)
    pub fn olive() -> Color {
        Color::from_rgb(128, 128, 0)
    }

    /// Navy (`#000080`)
    pub fn navy() -> Color {
        Color::from_rgb(0, 0, 128)
    }

    /// Purple (`#800080`)
    pub fn purple() -> Color {
        Color::from_rgb(128, 0, 128)
    }

    /// Teal (`#008080`)
    pub fn teal() -> Color {
        Color::from_rgb(0, 128, 128)
    }

    /// Silver (`#c0c0c0`)
    pub fn silver() -> Color {
        Color::from_rgb(192, 192, 192)
    }

    /// Gray (`#808080`)
    pub fn gray() -> Color {
        Color::from_rgb(128, 128, 128)
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

    /// Adjust the long-, medium-, and short-wavelength cone perception of a color to simulate what
    /// a colorblind person sees. Since there are multiple kinds of colorblindness, the desired
    /// kind must be specified in `cb_ty`.
    pub fn simulate_colorblindness(&self, cb_ty: ColorblindnessType) -> Color {
        // Coefficients here are taken from
        // https://ixora.io/projects/colorblindness/color-blindness-simulation-research/
        let (l, m, s, alpha) = match cb_ty {
            ColorblindnessType::Protanopia => {
                let LMS { m, s, alpha, .. } = self.to_lms();
                let l = 1.051_182_94 * m - 0.051_160_99 * s;
                (l, m, s, alpha)
            }
            ColorblindnessType::Deuteranopia => {
                let LMS { l, s, alpha, .. } = self.to_lms();
                let m = 0.951_309_2 * l + 0.048_669_92 * s;
                (l, m, s, alpha)
            }
            ColorblindnessType::Tritanopia => {
                let LMS { l, m, alpha, .. } = self.to_lms();
                let s = -0.867_447_36 * l + 1.867_270_89 * m;
                (l, m, s, alpha)
            }
        };

        Color::from_lms(l, m, s, alpha)
    }

    /// Convert a color to a gray tone with the same perceived luminance (see `luminance`).
    pub fn to_gray(&self) -> Color {
        let hue = self.hue;
        let c = self.to_lch();

        // the desaturation step is only needed to correct minor rounding errors.
        let mut gray = Color::from_lch(c.l, 0.0, 0.0, 1.0).desaturate(1.0);

        // Restore the hue value (does not alter the color, but makes it able to add saturation
        // again)
        gray.hue = hue;

        gray
    }

    /// The percieved brightness of the color (A number between 0.0 and 1.0).
    ///
    /// See: https://www.w3.org/TR/AERT#color-contrast
    pub fn brightness(&self) -> Scalar {
        let c = self.to_rgba_float();
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
        let c = self.to_rgba_float();
        let r = f(c.r);
        let g = f(c.g);
        let b = f(c.b);

        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// Contrast ratio between two colors as defined by the WCAG. The ratio can range from 1.0
    /// to 21.0. Two colors with a contrast ratio of 4.5 or higher can be used as text color and
    /// background color and should be well readable.
    ///
    /// https://www.w3.org/TR/2008/REC-WCAG20-20081211/#contrast-ratiodef
    pub fn contrast_ratio(&self, other: &Color) -> Scalar {
        let l_self = self.luminance();
        let l_other = other.luminance();

        if l_self > l_other {
            (l_self + 0.05) / (l_other + 0.05)
        } else {
            (l_other + 0.05) / (l_self + 0.05)
        }
    }

    /// Return a readable foreground text color (either `black` or `white`) for a
    /// given background color.
    pub fn text_color(&self) -> Color {
        // This threshold can be easily computed by solving
        //
        //   contrast(L_threshold, L_black) == contrast(L_threshold, L_white)
        //
        // where contrast(.., ..) is the color contrast as defined by the WCAG (see above)
        const THRESHOLD: Scalar = 0.179;

        if self.luminance() > THRESHOLD {
            Color::black()
        } else {
            Color::white()
        }
    }

    /// Compute the perceived 'distance' between two colors according to the CIE76 delta-E
    /// standard. A distance below ~2.3 is not noticable.
    ///
    /// See: https://en.wikipedia.org/wiki/Color_difference
    pub fn distance_delta_e_cie76(&self, other: &Color) -> Scalar {
        delta_e::cie76(&self.to_lab(), &other.to_lab())
    }

    /// Compute the perceived 'distance' between two colors according to the CIEDE2000 delta-E
    /// standard.
    ///
    /// See: https://en.wikipedia.org/wiki/Color_difference
    pub fn distance_delta_e_ciede2000(&self, other: &Color) -> Scalar {
        delta_e::ciede2000(&self.to_lab(), &other.to_lab())
    }

    /// Mix two colors by linearly interpolating between them in the specified color space.
    /// For the angle-like components (hue), the shortest path along the unit circle is chosen.
    pub fn mix<C: ColorSpace>(self: &Color, other: &Color, fraction: Fraction) -> Color {
        C::from_color(self)
            .mix(&C::from_color(other), fraction)
            .into_color()
    }
}

impl fmt::Debug for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Color::from_{}", self.to_rgb_string(Format::NoSpaces))
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

impl ColorSpace for RGBA<f64> {
    fn from_color(c: &Color) -> Self {
        c.to_rgba_float()
    }

    fn into_color(&self) -> Color {
        Color::from_rgba_float(self.r, self.g, self.b, self.alpha)
    }

    fn mix(&self, other: &Self, fraction: Fraction) -> Self {
        Self {
            r: interpolate(self.r, other.r, fraction),
            g: interpolate(self.g, other.g, fraction),
            b: interpolate(self.b, other.b, fraction),
            alpha: interpolate(self.alpha, other.alpha, fraction),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HSLA {
    pub h: Scalar,
    pub s: Scalar,
    pub l: Scalar,
    pub alpha: Scalar,
}

impl ColorSpace for HSLA {
    fn from_color(c: &Color) -> Self {
        c.to_hsla()
    }

    fn into_color(&self) -> Color {
        Color::from_hsla(self.h, self.s, self.l, self.alpha)
    }

    fn mix(&self, other: &Self, fraction: Fraction) -> Self {
        // make sure that the hue is preserved when mixing with gray colors
        let self_hue = if self.s < 0.0001 { other.h } else { self.h };
        let other_hue = if other.s < 0.0001 { self.h } else { other.h };

        Self {
            h: interpolate_angle(self_hue, other_hue, fraction),
            s: interpolate(self.s, other.s, fraction),
            l: interpolate(self.l, other.l, fraction),
            alpha: interpolate(self.alpha, other.alpha, fraction),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct XYZ {
    pub x: Scalar,
    pub y: Scalar,
    pub z: Scalar,
    pub alpha: Scalar,
}

/// A color space whose axes correspond to the responsivity spectra of the long-, medium-, and
/// short-wavelength cone cells in the human eye. More info
/// [here](https://en.wikipedia.org/wiki/LMS_color_space).
#[derive(Debug, Clone, PartialEq)]
pub struct LMS {
    pub l: Scalar,
    pub m: Scalar,
    pub s: Scalar,
    pub alpha: Scalar,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lab {
    pub l: Scalar,
    pub a: Scalar,
    pub b: Scalar,
    pub alpha: Scalar,
}

impl ColorSpace for Lab {
    fn from_color(c: &Color) -> Self {
        c.to_lab()
    }

    fn into_color(&self) -> Color {
        Color::from_lab(self.l, self.a, self.b, self.alpha)
    }

    fn mix(&self, other: &Self, fraction: Fraction) -> Self {
        Self {
            l: interpolate(self.l, other.l, fraction),
            a: interpolate(self.a, other.a, fraction),
            b: interpolate(self.b, other.b, fraction),
            alpha: interpolate(self.alpha, other.alpha, fraction),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LCh {
    pub l: Scalar,
    pub c: Scalar,
    pub h: Scalar,
    pub alpha: Scalar,
}

impl ColorSpace for LCh {
    fn from_color(c: &Color) -> Self {
        c.to_lch()
    }

    fn into_color(&self) -> Color {
        Color::from_lch(self.l, self.c, self.h, self.alpha)
    }

    fn mix(&self, other: &Self, fraction: Fraction) -> Self {
        // make sure that the hue is preserved when mixing with gray colors
        let self_hue = if self.c < 0.1 { other.h } else { self.h };
        let other_hue = if other.c < 0.1 { self.h } else { other.h };

        Self {
            l: interpolate(self.l, other.l, fraction),
            c: interpolate(self.c, other.c, fraction),
            h: interpolate_angle(self_hue, other_hue, fraction),
            alpha: interpolate(self.alpha, other.alpha, fraction),
        }
    }
}

/// A representation of the different kinds of colorblindness. More info
/// [here](https://en.wikipedia.org/wiki/Color_blindness).
pub enum ColorblindnessType {
    /// Protanopic people lack red cones
    Protanopia,
    /// Deuteranopic people lack green cones
    Deuteranopia,
    /// Tritanopic people lack blue cones
    Tritanopia,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    Spaces,
    NoSpaces,
}

/// The representation of a color stop for a `ColorScale`.
/// The position defines where the color is placed from left (0.0) to right (1.0).
#[derive(Debug, Clone)]
struct ColorStop {
    color: Color,
    position: Fraction,
}

/// The representation of a color scale.
/// The first `ColorStop` (position 0.0) defines the left end color.
/// The last `ColorStop` (position 1.0) defines the right end color.
#[derive(Debug, Clone)]
pub struct ColorScale {
    color_stops: Vec<ColorStop>,
}

impl ColorScale {
    /// Create an empty `ColorScale`.
    pub fn empty() -> Self {
        Self {
            color_stops: Vec::new(),
        }
    }

    /// Add a `Color` at the given position.
    pub fn add_stop(&mut self, color: Color, position: Fraction) -> &mut Self {
        #![allow(clippy::float_cmp)]
        let same_position = self
            .color_stops
            .iter_mut()
            .find(|c| position.value() == c.position.value());

        match same_position {
            Some(color_stop) => color_stop.color = color,
            None => {
                let next_index = self
                    .color_stops
                    .iter()
                    .position(|c| position.value() < c.position.value());

                let index = next_index.unwrap_or_else(|| self.color_stops.len());

                let color_stop = ColorStop { color, position };

                self.color_stops.insert(index, color_stop);
            }
        };

        self
    }

    /// Get the color at the given position using the mixing function.
    ///
    /// Note:
    /// - No color is returned if position isn't between two color stops or the `ColorScale` is empty.
    pub fn sample(
        &self,
        position: Fraction,
        mix: &dyn Fn(&Color, &Color, Fraction) -> Color,
    ) -> Option<Color> {
        if self.color_stops.len() < 2 {
            return None;
        }

        let left_stop = self
            .color_stops
            .iter()
            .rev()
            .find(|c| position.value() >= c.position.value());

        let right_stop = self
            .color_stops
            .iter()
            .find(|c| position.value() <= c.position.value());

        match (left_stop, right_stop) {
            (Some(left_stop), Some(right_stop)) => {
                let diff_color_stops = right_stop.position.value() - left_stop.position.value();
                let diff_position = position.value() - left_stop.position.value();
                let local_position = Fraction::from(diff_position / diff_color_stops);

                let color = mix(&left_stop.color, &right_stop.color, local_position);

                Some(color)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CMYK {
    pub c: Scalar,
    pub m: Scalar,
    pub y: Scalar,
    pub k: Scalar,
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
    fn color_partial_eq() {
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
    fn rgb_to_hsl_conversion() {
        assert_eq!(Color::white(), Color::from_rgb_float(1.0, 1.0, 1.0));
        assert_eq!(Color::gray(), Color::from_rgb_float(0.5, 0.5, 0.5));
        assert_eq!(Color::black(), Color::from_rgb_float(0.0, 0.0, 0.0));
        assert_eq!(Color::red(), Color::from_rgb_float(1.0, 0.0, 0.0));
        assert_eq!(
            Color::from_hsl(60.0, 1.0, 0.375),
            Color::from_rgb_float(0.75, 0.75, 0.0)
        ); //yellow-green
        assert_eq!(Color::green(), Color::from_rgb_float(0.0, 0.5, 0.0));
        assert_eq!(
            Color::from_hsl(240.0, 1.0, 0.75),
            Color::from_rgb_float(0.5, 0.5, 1.0)
        ); // blue-ish
        assert_eq!(
            Color::from_hsl(49.5, 0.893, 0.497),
            Color::from_rgb_float(0.941, 0.785, 0.053)
        ); // yellow
        assert_eq!(
            Color::from_hsl(162.4, 0.779, 0.447),
            Color::from_rgb_float(0.099, 0.795, 0.591)
        ); // cyan 2
    }

    #[test]
    fn rgb_roundtrip_conversion() {
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
    fn to_u32() {
        assert_eq!(0, Color::black().to_u32());
        assert_eq!(0xff0000, Color::red().to_u32());
        assert_eq!(0xffffff, Color::white().to_u32());
        assert_eq!(0xf4230f, Color::from_rgb(0xf4, 0x23, 0x0f).to_u32());
    }

    #[test]
    fn xyz_conversion() {
        assert_eq!(Color::white(), Color::from_xyz(0.9505, 1.0, 1.0890, 1.0));
        assert_eq!(Color::red(), Color::from_xyz(0.4123, 0.2126, 0.01933, 1.0));
        assert_eq!(
            Color::from_hsl(109.999, 0.08654, 0.407843),
            Color::from_xyz(0.13123, 0.15372, 0.13174, 1.0)
        );

        let roundtrip = |h, s, l| {
            let color1 = Color::from_hsl(h, s, l);
            let xyz1 = color1.to_xyz();
            let color2 = Color::from_xyz(xyz1.x, xyz1.y, xyz1.z, 1.0);
            assert_almost_equal(&color1, &color2);
        };

        for hue in 0..360 {
            roundtrip(Scalar::from(hue), 0.2, 0.8);
        }
    }

    #[test]
    fn lms_conversion() {
        let roundtrip = |h, s, l| {
            let color1 = Color::from_hsl(h, s, l);
            let lms1 = color1.to_lms();
            let color2 = Color::from_lms(lms1.l, lms1.m, lms1.s, 1.0);
            assert_almost_equal(&color1, &color2);
        };

        for hue in 0..360 {
            roundtrip(Scalar::from(hue), 0.2, 0.8);
        }
    }

    #[test]
    fn lab_conversion() {
        assert_eq!(Color::red(), Color::from_lab(53.233, 80.109, 67.22, 1.0));

        let roundtrip = |h, s, l| {
            let color1 = Color::from_hsl(h, s, l);
            let lab1 = color1.to_lab();
            let color2 = Color::from_lab(lab1.l, lab1.a, lab1.b, 1.0);
            assert_almost_equal(&color1, &color2);
        };

        for hue in 0..360 {
            roundtrip(Scalar::from(hue), 0.2, 0.8);
        }
    }

    #[test]
    fn lch_conversion() {
        assert_eq!(
            Color::from_hsl(0.0, 1.0, 0.245),
            Color::from_lch(24.829, 60.093, 38.18, 1.0)
        );

        let roundtrip = |h, s, l| {
            let color1 = Color::from_hsl(h, s, l);
            let lch1 = color1.to_lch();
            let color2 = Color::from_lch(lch1.l, lch1.c, lch1.h, 1.0);
            assert_almost_equal(&color1, &color2);
        };

        for hue in 0..360 {
            roundtrip(Scalar::from(hue), 0.2, 0.8);
        }
    }

    #[test]
    fn rotate_hue() {
        assert_eq!(Color::lime(), Color::red().rotate_hue(120.0));
    }

    #[test]
    fn complementary() {
        assert_eq!(Color::fuchsia(), Color::lime().complementary());
        assert_eq!(Color::lime(), Color::fuchsia().complementary());
    }

    #[test]
    fn lighten() {
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
    fn to_gray() {
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
    fn brightness() {
        assert_eq!(0.0, Color::black().brightness());
        assert_eq!(1.0, Color::white().brightness());
        assert_eq!(0.5, Color::graytone(0.5).brightness());
    }

    #[test]
    fn luminance() {
        assert_eq!(1.0, Color::white().luminance());
        let hotpink = Color::from_rgb(255, 105, 180);
        assert_relative_eq!(0.347, hotpink.luminance(), max_relative = 0.01);
        assert_eq!(0.0, Color::black().luminance());
    }

    #[test]
    fn contrast_ratio() {
        assert_relative_eq!(21.0, Color::black().contrast_ratio(&Color::white()));
        assert_relative_eq!(21.0, Color::white().contrast_ratio(&Color::black()));

        assert_relative_eq!(1.0, Color::white().contrast_ratio(&Color::white()));
        assert_relative_eq!(1.0, Color::red().contrast_ratio(&Color::red()));

        assert_relative_eq!(
            4.26,
            Color::from_rgb(255, 119, 153).contrast_ratio(&Color::from_rgb(0, 68, 85)),
            max_relative = 0.01
        );
    }

    #[test]
    fn text_color() {
        assert_eq!(Color::white(), Color::graytone(0.4).text_color());
        assert_eq!(Color::black(), Color::graytone(0.6).text_color());
    }

    #[test]
    fn distance_delta_e_cie76() {
        let c = Color::from_rgb(255, 127, 14);
        assert_eq!(0.0, c.distance_delta_e_cie76(&c));

        let c1 = Color::from_rgb(50, 100, 200);
        let c2 = Color::from_rgb(200, 10, 0);
        assert_eq!(123.0, c1.distance_delta_e_cie76(&c2).round());
    }

    #[test]
    fn to_hsl_string() {
        let c = Color::from_hsl(91.3, 0.541, 0.983);
        assert_eq!("hsl(91, 54.1%, 98.3%)", c.to_hsl_string(Format::Spaces));
    }

    #[test]
    fn to_rgb_string() {
        let c = Color::from_rgb(255, 127, 4);
        assert_eq!("rgb(255, 127, 4)", c.to_rgb_string(Format::Spaces));
    }

    #[test]
    fn to_rgb_float_string() {
        assert_eq!(
            "rgb(0.000, 0.000, 0.000)",
            Color::black().to_rgb_float_string(Format::Spaces)
        );

        assert_eq!(
            "rgb(1.000, 1.000, 1.000)",
            Color::white().to_rgb_float_string(Format::Spaces)
        );

        // some minor rounding errors here, but that is to be expected:
        let c = Color::from_rgb_float(0.12, 0.45, 0.78);
        assert_eq!(
            "rgb(0.122, 0.451, 0.780)",
            c.to_rgb_float_string(Format::Spaces)
        );
    }

    #[test]
    fn to_rgb_hex_string() {
        let c = Color::from_rgb(255, 127, 4);
        assert_eq!("ff7f04", c.to_rgb_hex_string(false));
        assert_eq!("#ff7f04", c.to_rgb_hex_string(true));
    }

    #[test]
    fn to_lab_string() {
        let c = Color::from_lab(41.0, 83.0, -93.0, 1.0);
        assert_eq!("Lab(41, 83, -93)", c.to_lab_string(Format::Spaces));
    }

    #[test]
    fn to_lch_string() {
        let c = Color::from_lch(52.0, 44.0, 271.0, 1.0);
        assert_eq!("LCh(52, 44, 271)", c.to_lch_string(Format::Spaces));
    }

    #[test]
    fn mix() {
        assert_eq!(
            Color::purple(),
            Color::red().mix::<RGBA<f64>>(&Color::blue(), Fraction::from(0.5))
        );
        assert_eq!(
            Color::fuchsia(),
            Color::red().mix::<HSLA>(&Color::blue(), Fraction::from(0.5))
        );
    }

    #[test]
    fn mixing_with_gray_preserves_hue() {
        let hue = 123.0;

        let input = Color::from_hsla(hue, 0.5, 0.5, 1.0);

        let hue_after_mixing = |other| input.mix::<HSLA>(&other, Fraction::from(0.5)).to_hsla().h;

        assert_eq!(hue, hue_after_mixing(Color::black()));
        assert_eq!(hue, hue_after_mixing(Color::graytone(0.2)));
        assert_eq!(hue, hue_after_mixing(Color::graytone(0.7)));
        assert_eq!(hue, hue_after_mixing(Color::white()));
    }

    #[test]
    fn color_scale_add_preserves_ordering() {
        let mut color_scale = ColorScale::empty();

        color_scale
            .add_stop(Color::red(), Fraction::from(0.5))
            .add_stop(Color::gray(), Fraction::from(0.0))
            .add_stop(Color::blue(), Fraction::from(1.0));

        assert_eq!(color_scale.color_stops.get(0).unwrap().color, Color::gray());
        assert_eq!(color_scale.color_stops.get(1).unwrap().color, Color::red());
        assert_eq!(color_scale.color_stops.get(2).unwrap().color, Color::blue());
    }

    #[test]
    fn color_scale_empty_sample_none() {
        let mix = Color::mix::<Lab>;

        let color_scale = ColorScale::empty();

        let color = color_scale.sample(Fraction::from(0.0), &mix);

        assert_eq!(color, None);
    }

    #[test]
    fn color_scale_one_color_sample_none() {
        let mix = Color::mix::<Lab>;

        let mut color_scale = ColorScale::empty();

        color_scale.add_stop(Color::red(), Fraction::from(0.0));

        let color = color_scale.sample(Fraction::from(0.0), &mix);

        assert_eq!(color, None);
    }

    #[test]
    fn color_scale_sample_same_position() {
        let mix = Color::mix::<Lab>;

        let mut color_scale = ColorScale::empty();

        color_scale
            .add_stop(Color::red(), Fraction::from(0.0))
            .add_stop(Color::green(), Fraction::from(1.0))
            .add_stop(Color::blue(), Fraction::from(0.0))
            .add_stop(Color::white(), Fraction::from(1.0));

        let sample_blue = color_scale.sample(Fraction::from(0.0), &mix).unwrap();
        let sample_white = color_scale.sample(Fraction::from(1.0), &mix).unwrap();

        assert_eq!(sample_blue, Color::blue());
        assert_eq!(sample_white, Color::white());
    }

    #[test]
    fn color_scale_sample() {
        let mix = Color::mix::<Lab>;

        let mut color_scale = ColorScale::empty();

        color_scale
            .add_stop(Color::green(), Fraction::from(1.0))
            .add_stop(Color::red(), Fraction::from(0.0));

        let sample_red_green = color_scale.sample(Fraction::from(0.5), &mix).unwrap();

        let mix_red_green = mix(&Color::red(), &Color::green(), Fraction::from(0.5));

        assert_eq!(sample_red_green, mix_red_green);
    }

    #[test]
    fn color_scale_sample_position() {
        let mix = Color::mix::<Lab>;

        let mut color_scale = ColorScale::empty();

        color_scale
            .add_stop(Color::green(), Fraction::from(0.5))
            .add_stop(Color::red(), Fraction::from(0.0))
            .add_stop(Color::blue(), Fraction::from(1.0));

        let sample_red = color_scale.sample(Fraction::from(0.0), &mix).unwrap();
        let sample_green = color_scale.sample(Fraction::from(0.5), &mix).unwrap();
        let sample_blue = color_scale.sample(Fraction::from(1.0), &mix).unwrap();

        let sample_red_green = color_scale.sample(Fraction::from(0.25), &mix).unwrap();
        let sample_green_blue = color_scale.sample(Fraction::from(0.75), &mix).unwrap();

        let mix_red_green = mix(&Color::red(), &Color::green(), Fraction::from(0.50));
        let mix_green_blue = mix(&Color::green(), &Color::blue(), Fraction::from(0.50));

        assert_eq!(sample_red, Color::red());
        assert_eq!(sample_green, Color::green());
        assert_eq!(sample_blue, Color::blue());

        assert_eq!(sample_red_green, mix_red_green);
        assert_eq!(sample_green_blue, mix_green_blue);
    }

    #[test]
    fn to_cmyk_string() {
        let white = Color::from_rgb(255, 255, 255);
        assert_eq!("cmyk(0, 0, 0, 0)", white.to_cmyk_string(Format::Spaces));

        let black = Color::from_rgb(0, 0, 0);
        assert_eq!("cmyk(0, 0, 0, 100)", black.to_cmyk_string(Format::Spaces));

        let c = Color::from_rgb(19, 19, 1);
        assert_eq!("cmyk(0, 0, 95, 93)", c.to_cmyk_string(Format::Spaces));

        let c1 = Color::from_rgb(55, 55, 55);
        assert_eq!("cmyk(0, 0, 0, 78)", c1.to_cmyk_string(Format::Spaces));

        let c2 = Color::from_rgb(136, 117, 78);
        assert_eq!("cmyk(0, 14, 43, 47)", c2.to_cmyk_string(Format::Spaces));

        let c3 = Color::from_rgb(143, 111, 76);
        assert_eq!("cmyk(0, 22, 47, 44)", c3.to_cmyk_string(Format::Spaces));
    }
}
