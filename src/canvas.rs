use ansi_term::{Colour, Style};

#[derive(Debug, Clone)]
pub enum Pixel {
    Empty,
    Char(char),
    Color(Colour),
}

pub struct Canvas {
    height: usize,
    width: usize,
    pixels: Vec<Pixel>,
}

impl Canvas {
    pub fn new(height: usize, width: usize) -> Self {
        let mut pixels = vec![];
        pixels.resize(height * width, Pixel::Empty);
        Canvas {
            height,
            width,
            pixels,
        }
    }

    pub fn draw_rect(
        &mut self,
        row: usize,
        col: usize,
        height: usize,
        width: usize,
        color: Colour,
    ) {
        for i in 0..height {
            for j in 0..width {
                *self.pixel_mut(row + i, col + j) = Pixel::Color(color);
            }
        }
    }

    pub fn draw_checkerboard(
        &mut self,
        row: usize,
        col: usize,
        height: usize,
        width: usize,
        dark: Colour,
        light: Colour,
    ) {
        for i in 0..height {
            for j in 0..width {
                let color = if (i + j / 2) % 2 == 0 { dark } else { light };
                *self.pixel_mut(row + i, col + j) = Pixel::Color(color);
            }
        }
    }

    pub fn draw_text(&mut self, row: usize, col: usize, text: &str) {
        let mut j = 0;
        for c in text.chars() {
            *self.pixel_mut(row, col + j) = Pixel::Char(c);
            j += 1;
        }
    }

    pub fn print(&self) {
        for i in 0..self.height {
            for j in 0..self.width {
                match self.pixel(i, j) {
                    Pixel::Empty => print!(" "),
                    Pixel::Color(color) => print!("{}", Style::new().on(*color).paint(" ")),
                    Pixel::Char(c) => print!("{}", c),
                }
            }
            println!();
        }
    }

    fn pixel(&self, i: usize, j: usize) -> &Pixel {
        assert!(i < self.height);
        assert!(j < self.width);
        &self.pixels[i * self.width + j]
    }

    fn pixel_mut(&mut self, i: usize, j: usize) -> &mut Pixel {
        assert!(i < self.height);
        assert!(j < self.width);
        &mut self.pixels[i * self.width + j]
    }
}
