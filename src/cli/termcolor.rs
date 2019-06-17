use ansi_term::Color as TermColor;
use pastel::Color;

pub fn to_termcolor(c: &Color) -> TermColor {
    let rgba = c.to_rgba();
    TermColor::RGB(rgba.r, rgba.g, rgba.b)
}
