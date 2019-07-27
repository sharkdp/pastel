use pastel::ansi::Brush;

pub struct Config {
    pub padding: usize,
    pub colorpicker_width: usize,
    pub interactive_mode: bool,
    pub brush: Brush,
}
