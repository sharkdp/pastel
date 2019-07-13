use ansi_term::Color as TermColor;
use pastel::Color;

pub trait ToTermColor {
    fn to_termcolor(&self) -> TermColor;
}

impl ToTermColor for Color {
    fn to_termcolor(&self) -> TermColor {
        let rgba = self.to_rgba();
        TermColor::RGB(rgba.r, rgba.g, rgba.b)
    }
}
