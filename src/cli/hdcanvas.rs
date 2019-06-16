use ansi_term::Color as TermColor;

pub struct Canvas {
    height: usize,
    width: usize,
    pixels: Vec<Option<TermColor>>,
    chars: Vec<Option<char>>,
}

impl Canvas {
    pub fn new(height: usize, width: usize) -> Self {
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
        }
    }

    pub fn draw_rect(
        &mut self,
        row: usize,
        col: usize,
        height: usize,
        width: usize,
        color: TermColor,
    ) {
        for i in 0..height {
            for j in 0..width {
                *self.pixel_mut(row + i, col + j) = Some(color);
            }
        }
    }

    pub fn draw_checkerboard(
        &mut self,
        row: usize,
        col: usize,
        height: usize,
        width: usize,
        dark: TermColor,
        light: TermColor,
    ) {
        for i in 0..height {
            for j in 0..width {
                let color = if (i + j) % 2 == 0 { dark } else { light };
                *self.pixel_mut(row + i, col + j) = Some(color);
            }
        }
    }

    pub fn draw_text(&mut self, row: usize, col: usize, text: &str) {
        assert!(row % 2 == 0);

        for (j, c) in text.chars().enumerate() {
            *self.char_mut(row / 2, col + j) = Some(c);
        }
    }

    pub fn print(&self) {
        for i_div_2 in 0..self.height / 2 {
            for j in 0..self.width {
                if let Some(c) = self.char(i_div_2, j) {
                    print!("{}", c);
                } else {
                    let p_top = self.pixel(2 * i_div_2, j);
                    let p_bottom = self.pixel(2 * i_div_2 + 1, j);

                    match (p_top, p_bottom) {
                        (Some(top), Some(bottom)) => print!("{}", top.on(*bottom).paint("▀")),
                        (Some(top), None) => print!("{}", top.paint("▀")),
                        (None, Some(bottom)) => print!("{}", bottom.paint("▄")),
                        (None, None) => print!(" "),
                    }
                }
            }
            println!();
        }
    }

    fn pixel(&self, i: usize, j: usize) -> &Option<TermColor> {
        assert!(i < self.height);
        assert!(j < self.width);
        &self.pixels[i * self.width + j]
    }

    fn pixel_mut(&mut self, i: usize, j: usize) -> &mut Option<TermColor> {
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
