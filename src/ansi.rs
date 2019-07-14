use crate::Color;

fn cube_to_8bit(code: u8) -> u8 {
    assert!(code < 6);
    match code {
        0 => 0,
        _ => 55 + 40 * code,
    }
}

pub trait AnsiColor {
    fn from_ansi_8bit(code: u8) -> Self;
    fn to_ansi_8bit(&self) -> u8;

    fn to_ansi_sequence_8bit(&self) -> String;
    fn to_ansi_sequence_24bit(&self) -> String;
}

impl AnsiColor for Color {
    /// Create a color from an 8-bit ANSI escape code
    ///
    /// See: https://en.wikipedia.org/wiki/ANSI_escape_code
    fn from_ansi_8bit(code: u8) -> Color {
        match code {
            0 => Color::black(),
            1 => Color::maroon(),
            2 => Color::green(),
            3 => Color::olive(),
            4 => Color::navy(),
            5 => Color::purple(),
            6 => Color::teal(),
            7 => Color::silver(),
            8 => Color::gray(),
            9 => Color::red(),
            10 => Color::lime(),
            11 => Color::yellow(),
            12 => Color::blue(),
            13 => Color::fuchsia(),
            14 => Color::aqua(),
            15 => Color::white(),
            16..=231 => {
                // 6 x 6 x 6 cube of 216 colors. We need to decode from
                //
                //    code = 16 + 36 × r + 6 × g + b

                let code_rgb = code - 16;
                let blue = code_rgb % 6;

                let code_rg = (code_rgb - blue) / 6;
                let green = code_rg % 6;

                let red = (code_rg - green) / 6;

                Color::from_rgb(cube_to_8bit(red), cube_to_8bit(green), cube_to_8bit(blue))
            }
            232..=255 => {
                // grayscale from (almost) black to (almost) white in 24 steps

                let gray_value = 10 * (code - 232) + 8;
                Color::from_rgb(gray_value, gray_value, gray_value)
            }
        }
    }

    /// Approximate a color by its closest 8-bit ANSI color (as measured by the perceived
    /// color distance).
    ///
    /// See: https://en.wikipedia.org/wiki/ANSI_escape_code
    fn to_ansi_8bit(&self) -> u8 {
        let mut codes: Vec<u8> = (0..255).collect();
        codes.sort_by_key(|code| self.distance(&Color::from_ansi_8bit(*code)) as i32);

        codes[0]
    }

    /// Return an ANSI escape sequence in 8-bit representation:
    /// `ESC[38;5;CODEm`, where CODE represents the color.
    fn to_ansi_sequence_8bit(&self) -> String {
        format!("\x1b[38;5;{}m", self.to_ansi_8bit())
    }

    /// Return an ANSI escape sequence in 24-bit representation:
    /// `ESC[38;2;R;G;Bm`, where CODE represents the color.
    fn to_ansi_sequence_24bit(&self) -> String {
        let rgba = self.to_rgba();
        format!("\x1b[38;2;{r};{g};{b}m", r = rgba.r, g = rgba.g, b = rgba.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ansi_8bit_lower_16() {
        assert_eq!(Color::black(), Color::from_ansi_8bit(0));
        assert_eq!(Color::red(), Color::from_ansi_8bit(9));
        assert_eq!(Color::white(), Color::from_ansi_8bit(15));
    }

    #[test]
    fn test_from_ansi_8bit_cube() {
        assert_eq!(Color::black(), Color::from_ansi_8bit(16));
        assert_eq!(Color::from_rgb(0, 0, 95), Color::from_ansi_8bit(17));
        assert_eq!(Color::from_rgb(95, 175, 135), Color::from_ansi_8bit(72));
        assert_eq!(Color::from_rgb(255, 215, 95), Color::from_ansi_8bit(221));
        assert_eq!(Color::white(), Color::from_ansi_8bit(231));
    }

    #[test]
    fn test_from_ansi_8bit_grays() {
        assert_eq!(Color::from_rgb(8, 8, 8), Color::from_ansi_8bit(232));
        assert_eq!(Color::from_rgb(108, 108, 108), Color::from_ansi_8bit(242));
        assert_eq!(Color::from_rgb(238, 238, 238), Color::from_ansi_8bit(255));
    }

    #[test]
    fn test_to_ansi_8bit_lower_16() {
        assert_eq!(0, Color::black().to_ansi_8bit());
        assert_eq!(1, Color::maroon().to_ansi_8bit());
        assert_eq!(2, Color::green().to_ansi_8bit());
        assert_eq!(3, Color::olive().to_ansi_8bit());
        assert_eq!(4, Color::navy().to_ansi_8bit());
        assert_eq!(5, Color::purple().to_ansi_8bit());
        assert_eq!(6, Color::teal().to_ansi_8bit());
        assert_eq!(7, Color::silver().to_ansi_8bit());
        assert_eq!(8, Color::gray().to_ansi_8bit());
        assert_eq!(9, Color::red().to_ansi_8bit());
        assert_eq!(10, Color::lime().to_ansi_8bit());
        assert_eq!(11, Color::yellow().to_ansi_8bit());
        assert_eq!(12, Color::blue().to_ansi_8bit());
        assert_eq!(13, Color::fuchsia().to_ansi_8bit());
        assert_eq!(14, Color::aqua().to_ansi_8bit());
        assert_eq!(15, Color::white().to_ansi_8bit());

        assert_eq!(0, Color::black().lighten(0.01).to_ansi_8bit());
    }

    #[test]
    fn test_to_ansi_8bit_cube() {
        assert_eq!(72, Color::from_rgb(95, 175, 135).to_ansi_8bit());
        assert_eq!(221, Color::from_rgb(255, 215, 95).to_ansi_8bit());
    }

    #[test]
    fn test_to_ansi_8bit_grays() {
        assert_eq!(232, Color::from_rgb(8, 8, 8).to_ansi_8bit());
        assert_eq!(242, Color::from_rgb(108, 108, 108).to_ansi_8bit());
    }
}
