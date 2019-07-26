use std::borrow::Borrow;

use crate::Color;

use atty::{self, Stream};

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Ansi8Bit,
    TrueColor,
}

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

    fn to_ansi_sequence(&self, mode: Mode) -> String;
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

    /// Return an ANSI escape sequence in 8-bit or 24-bit representation:
    /// * 8-bit: `ESC[38;5;CODEm`, where CODE represents the color.
    /// * 24-bit: `ESC[38;2;R;G;Bm`, where R, G, B represent 8-bit RGB values
    fn to_ansi_sequence(&self, mode: Mode) -> String {
        match mode {
            Mode::Ansi8Bit => format!("\x1b[38;5;{}m", self.to_ansi_8bit()),
            Mode::TrueColor => {
                let rgba = self.to_rgba();
                format!("\x1b[38;2;{r};{g};{b}m", r = rgba.r, g = rgba.g, b = rgba.b)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    foreground: Option<Color>,
    background: Option<Color>,
    bold: bool,
    italic: bool,
    underline: bool,
}

impl Style {
    pub fn from_color(color: Color) -> Style {
        Style {
            foreground: Some(color),
            background: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }

    pub fn foreground(&mut self, color: &Color) -> &mut Self {
        self.foreground = Some(color.clone());
        self
    }

    pub fn on<C: Borrow<Color>>(&mut self, color: C) -> &mut Self {
        self.background = Some(color.borrow().clone());
        self
    }

    pub fn bold(&mut self, on: bool) -> &mut Self {
        self.bold = on;
        self
    }

    pub fn italic(&mut self, on: bool) -> &mut Self {
        self.italic = on;
        self
    }

    pub fn underline(&mut self, on: bool) -> &mut Self {
        self.underline = on;
        self
    }

    pub fn escape_sequence(&self, mode: Mode) -> String {
        let mut codes: Vec<u8> = vec![];

        if let Some(ref fg) = self.foreground {
            match mode {
                Mode::Ansi8Bit => codes.extend_from_slice(&[38, 5, fg.to_ansi_8bit()]),
                Mode::TrueColor => {
                    let rgb = fg.to_rgba();
                    codes.extend_from_slice(&[38, 2, rgb.r, rgb.g, rgb.b]);
                }
            }
        }
        if let Some(ref bg) = self.background {
            let rgb = bg.to_rgba();
            codes.extend_from_slice(&[48, 2, rgb.r, rgb.g, rgb.b]);
        }

        if self.bold {
            codes.push(1);
        }

        if self.italic {
            codes.push(3);
        }

        if self.underline {
            codes.push(4);
        }

        if codes.is_empty() {
            codes.push(0);
        }

        format!(
            "\x1b[{codes}m",
            codes = codes
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(";")
        )
    }
}

impl Default for Style {
    fn default() -> Style {
        Style {
            foreground: None,
            background: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

pub trait ToAnsiStyle {
    fn ansi_style(&self) -> Style;
}

impl ToAnsiStyle for Color {
    fn ansi_style(&self) -> Style {
        Style::from_color(self.clone())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Brush {
    mode: Option<Mode>,
}

impl Brush {
    pub fn from_mode(mode: Mode) -> Self {
        Brush { mode: Some(mode) }
    }

    pub fn from_environment() -> Self {
        let mode = if atty::is(Stream::Stdout) {
            if std::env::var("COLORTERM") == Ok("truecolor".into()) {
                Some(Mode::TrueColor)
            } else {
                Some(Mode::Ansi8Bit)
            }
        } else {
            None
        };

        Brush { mode }
    }

    pub fn paint<S: AsRef<str>, T: Borrow<Style>>(&self, text: S, style: T) -> String {
        if let Some(ansi_mode) = self.mode {
            format!(
                "{begin}{text}{end}",
                begin = style.borrow().escape_sequence(ansi_mode),
                text = text.as_ref(),
                end = "\x1b[0m"
            )
        } else {
            format!("{}", text.as_ref())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_ansi_8bit_lower_16() {
        assert_eq!(Color::black(), Color::from_ansi_8bit(0));
        assert_eq!(Color::red(), Color::from_ansi_8bit(9));
        assert_eq!(Color::white(), Color::from_ansi_8bit(15));
    }

    #[test]
    fn from_ansi_8bit_cube() {
        assert_eq!(Color::black(), Color::from_ansi_8bit(16));
        assert_eq!(Color::from_rgb(0, 0, 95), Color::from_ansi_8bit(17));
        assert_eq!(Color::from_rgb(95, 175, 135), Color::from_ansi_8bit(72));
        assert_eq!(Color::from_rgb(255, 215, 95), Color::from_ansi_8bit(221));
        assert_eq!(Color::white(), Color::from_ansi_8bit(231));
    }

    #[test]
    fn from_ansi_8bit_grays() {
        assert_eq!(Color::from_rgb(8, 8, 8), Color::from_ansi_8bit(232));
        assert_eq!(Color::from_rgb(108, 108, 108), Color::from_ansi_8bit(242));
        assert_eq!(Color::from_rgb(238, 238, 238), Color::from_ansi_8bit(255));
    }

    #[test]
    fn to_ansi_8bit_lower_16() {
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
    fn to_ansi_8bit_cube() {
        assert_eq!(72, Color::from_rgb(95, 175, 135).to_ansi_8bit());
        assert_eq!(221, Color::from_rgb(255, 215, 95).to_ansi_8bit());
    }

    #[test]
    fn to_ansi_8bit_grays() {
        assert_eq!(232, Color::from_rgb(8, 8, 8).to_ansi_8bit());
        assert_eq!(242, Color::from_rgb(108, 108, 108).to_ansi_8bit());
    }

    #[test]
    fn ansi_style() {
        assert_eq!("\x1b[0m", Style::default().escape_sequence(Mode::TrueColor));

        assert_eq!(
            "\x1b[38;2;255;0;0m",
            Color::red().ansi_style().escape_sequence(Mode::TrueColor)
        );

        assert_eq!(
            "\x1b[38;5;9m",
            Color::red().ansi_style().escape_sequence(Mode::Ansi8Bit)
        );

        assert_eq!(
            "\x1b[38;2;255;0;0;48;2;0;0;255;1;3;4m",
            Color::red()
                .ansi_style()
                .on(Color::blue())
                .bold(true)
                .italic(true)
                .underline(true)
                .escape_sequence(Mode::TrueColor)
        );
    }

    #[test]
    fn brush() {
        let ansi = Brush::from_mode(Mode::TrueColor);

        assert_eq!(
            "\x1b[38;2;255;0;0;1mhello\x1b[0m",
            ansi.paint("hello", Color::red().ansi_style().bold(true))
        );
    }
}
