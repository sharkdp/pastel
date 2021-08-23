use pastel::named::{NamedColor, NAMED_COLORS};
use pastel::Color;

/// Returns a list of named colors, sorted by the perceived distance to the given color
pub fn similar_colors(color: &Color) -> Vec<&NamedColor> {
    let mut colors: Vec<&NamedColor> = NAMED_COLORS.iter().collect();
    colors.sort_by_key(|nc| (1000.0 * nc.color.distance_delta_e_ciede2000(color)) as i32);
    colors.dedup_by(|n1, n2| n1.color == n2.color);
    colors
}
