use pastel::ansi::Brush;

#[derive(Debug, Clone)]
pub struct Config<'p> {
    pub padding: usize,
    pub colorpicker_width: usize,
    pub colorcheck_width: usize,
    pub colorpicker: Option<&'p str>,
    pub interactive_mode: bool,
    pub brush: Brush,
}
