use pastel::ansi::Brush;

#[derive(Debug, Clone)]
pub struct Config {
    pub padding: usize,
    pub colorpicker_width: usize,
    pub colorcheck_width: usize,
    pub interactive_mode: bool,
    pub brush: Brush,
}
