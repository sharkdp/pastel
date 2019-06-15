use ansi_term::{Colour, Style};

#[derive(Debug, Clone)]
pub enum Pixel {
    Empty,
    Chars(char, char),
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
                let color = if (i + j) % 2 == 0 { dark } else { light };
                *self.pixel_mut(row + i, col + j) = Pixel::Color(color);
            }
        }
    }

    pub fn draw_text(&mut self, row: usize, col: usize, text: &str) {
        let mut j = 0;
        let mut chars = text.chars().peekable();

        while let Some(c1) = chars.next() {
            let c = chars.peek();
            let c2 = if let Some(c) = c { *c } else { ' ' };
            *self.pixel_mut(row, col + j) = Pixel::Chars(c1, c2);
            j += 1;
            chars.next();
        }
    }

    pub fn print(&self) {
        for i in 0..self.height {
            for j in 0..self.width {
                match self.pixel(i, j) {
                    Pixel::Empty => print!("  "),
                    Pixel::Color(color) => print!("{}", Style::new().on(*color).paint("  ")),
                    Pixel::Chars(c1, c2) => print!("{}{}", c1, c2),
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
