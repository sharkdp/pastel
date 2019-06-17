use pastel::Color;

use crate::x11colors::{NamedColor, X11_COLORS};

/// Returns a list of named colors, sorted by the perceived distance to the given color
pub fn similar_colors(color: &Color) -> Vec<&NamedColor> {
    let mut colors: Vec<&NamedColor> = X11_COLORS.iter().map(|r| r).collect();
    colors.sort_by_key(|nc| nc.color.distance(&color) as i32);
    colors.dedup_by(|n1, n2| n1.color == n2.color);
    colors
}
