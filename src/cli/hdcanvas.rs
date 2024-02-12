use std::io::Write;

use pastel::ansi::{Brush, ToAnsiStyle};
use pastel::Color;

use crate::Result;

pub struct Canvas {
    height: usize,
    width: usize,
    pixels: Vec<Option<Color>>,
    chars: Vec<Option<char>>,
    brush: Brush,
}

impl Canvas {
    pub fn new(height: usize, width: usize, brush: Brush) -> Self {
        assert!(height % 2 == 0);

        let mut pixels = vec![];
        pixels.resize(height * width, None);
        let mut chars = vec![];
        chars.resize(height / 2 * width, None);

        Canvas {
            height,
            width,
            pixels,
            chars,
            brush,
        }
    }

    pub fn draw_rect(
        &mut self,
        row: usize,
        col: usize,
        height: usize,
        width: usize,
        color: &Color,
    ) {
        for i in 0..height {
            for j in 0..width {
                let px = self.pixel_mut(row + i, col + j);
                *px = Some(match px {
                    Some(backdrop) => backdrop.composite(color),
                    None => color.clone(),
                });
            }
        }
    }

    pub fn draw_checkerboard(
        &mut self,
        row: usize,
        col: usize,
        height: usize,
        width: usize,
        dark: &Color,
        light: &Color,
    ) {
        for i in 0..height {
            for j in 0..width {
                let color = if (i + j) % 2 == 0 { dark } else { light };
                *self.pixel_mut(row + i, col + j) = Some(color.clone());
            }
        }
    }

    pub fn draw_text(&mut self, row: usize, col: usize, text: &str) {
        assert!(row % 2 == 0);

        for (j, c) in text.chars().enumerate() {
            *self.char_mut(row / 2, col + j) = Some(c);
        }
    }

    // The kitty terminal has a feature text_fg_override_threshold that
    // checks the difference in luminosity between text and background and
    // changes the text to black or white to make it readable if the
    // luminosity difference percentage is below the specified threshold.
    // Using block characters for graphics display can trigger this, causing
    // black or white lines or blocks, if the color is the same or too close.
    // The checkerboard should be ok unless the threshold is set fairly high.
    pub fn print(&self, out: &mut dyn Write) -> Result<()> {
        for i_div_2 in 0..self.height / 2 {
            for j in 0..self.width {
                if let Some(c) = self.char(i_div_2, j) {
                    write!(out, "{}", c)?;
                } else {
                    let p_top = self.pixel(2 * i_div_2, j);
                    let p_bottom = self.pixel(2 * i_div_2 + 1, j);

                    match (p_top, p_bottom) {
                        (Some(top), Some(bottom)) => {
                            if top == bottom {
                                write!(
                                    out,
                                    "{}",
                                    self.brush.paint(" ", top.ansi_style().on(bottom))
                                )?
                            } else {
                                write!(
                                    out,
                                    "{}",
                                    self.brush.paint("▀", top.ansi_style().on(bottom))
                                )?
                            }
                        }
                        (Some(top), None) => write!(out, "{}", self.brush.paint("▀", top))?,
                        (None, Some(bottom)) => write!(out, "{}", self.brush.paint("▄", bottom))?,
                        (None, None) => write!(out, " ")?,
                    };
                }
            }
            writeln!(out)?;
        }

        Ok(())
    }

    fn pixel(&self, i: usize, j: usize) -> &Option<Color> {
        assert!(i < self.height);
        assert!(j < self.width);
        &self.pixels[i * self.width + j]
    }

    fn pixel_mut(&mut self, i: usize, j: usize) -> &mut Option<Color> {
        assert!(i < self.height);
        assert!(j < self.width);
        &mut self.pixels[i * self.width + j]
    }

    fn char(&self, i: usize, j: usize) -> &Option<char> {
        assert!(i < self.height / 2);
        assert!(j < self.width);
        &self.chars[i * self.width + j]
    }

    fn char_mut(&mut self, i: usize, j: usize) -> &mut Option<char> {
        assert!(i < self.height / 2);
        assert!(j < self.width);
        &mut self.chars[i * self.width + j]
    }
}
