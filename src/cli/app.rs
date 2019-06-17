use ansi_term::Color as TermColor;
use atty::Stream;

use pastel::Color;

use crate::hdcanvas::Canvas;
use crate::termcolor::to_termcolor;
use crate::x11colors::{NamedColor, X11_COLORS};

/// Returns a list of named colors, sorted by the perceived distance to the given color
fn similar_colors(color: &Color) -> Vec<&NamedColor> {
    let mut colors: Vec<&NamedColor> = X11_COLORS.iter().map(|r| r).collect();
    colors.sort_by_key(|nc| nc.color.distance(&color) as i32);
    colors.dedup_by(|n1, n2| n1.color == n2.color);
    colors
}

pub fn show_color_tty(color: Color) {
    let rgba = color.to_rgba();
    let hsla = color.to_hsla();
    let terminal_color = to_termcolor(&color);

    const PADDING: usize = 2;
    const CHECKERBOARD_SIZE: usize = 20;
    const COLOR_PANEL_SIZE: usize = 14;

    const COLOR_PANEL_POSITION: usize = PADDING + (CHECKERBOARD_SIZE - COLOR_PANEL_SIZE) / 2;
    const TEXT_POSITION_X: usize = CHECKERBOARD_SIZE + 2 * PADDING;
    const TEXT_POSITION_Y: usize = PADDING + 2;

    let mut canvas = Canvas::new(2 * PADDING + CHECKERBOARD_SIZE, 55);
    canvas.draw_checkerboard(
        PADDING,
        PADDING,
        CHECKERBOARD_SIZE,
        CHECKERBOARD_SIZE,
        TermColor::RGB(240, 240, 240),
        TermColor::RGB(180, 180, 180),
    );
    canvas.draw_rect(
        COLOR_PANEL_POSITION,
        COLOR_PANEL_POSITION,
        COLOR_PANEL_SIZE,
        COLOR_PANEL_SIZE,
        terminal_color,
    );

    canvas.draw_text(
        TEXT_POSITION_Y + 0,
        TEXT_POSITION_X,
        &format!("Hex: #{:02x}{:02x}{:02x}", rgba.r, rgba.g, rgba.b),
    );
    canvas.draw_text(
        TEXT_POSITION_Y + 2,
        TEXT_POSITION_X,
        &format!("RGB: rgb({},{},{})", rgba.r, rgba.g, rgba.b),
    );
    canvas.draw_text(
        TEXT_POSITION_Y + 4,
        TEXT_POSITION_X,
        &format!(
            "HSL: hsl({:.0},{:.0}%,{:.0}%)",
            hsla.h,
            100.0 * hsla.s,
            100.0 * hsla.l
        ),
    );
    canvas.draw_text(TEXT_POSITION_Y + 8, TEXT_POSITION_X, "Most similar:");
    let similar = similar_colors(&color);
    for (i, nc) in similar.iter().enumerate().take(3) {
        canvas.draw_text(TEXT_POSITION_Y + 10 + 2 * i, TEXT_POSITION_X + 7, nc.name);
        canvas.draw_rect(
            TEXT_POSITION_Y + 10 + 2 * i,
            TEXT_POSITION_X + 1,
            2,
            5,
            to_termcolor(&nc.color),
        );
    }

    canvas.print();
}
