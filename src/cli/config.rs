use pastel::ansi::Brush;

pub struct Config {
    pub padding: usize,
    pub colorpicker_width: usize,
    pub interactive_mode: bool,
    pub brush: Brush,
}

impl Config {
    pub fn new(interactive_mode: bool) -> Config {
        Config {
            padding: 2,
            colorpicker_width: 40,
            interactive_mode,
            brush: Brush::from_environment(),
        }
    }
}
