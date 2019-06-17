use ansi_term::Color as TermColor;
use atty::Stream;

use pastel::Color;

use crate::hdcanvas::Canvas;
use crate::termcolor::to_termcolor;
use crate::utility::similar_colors;
use crate::x11colors::{NamedColor, X11_COLORS};

pub struct App {}

impl App {
    pub fn new() -> App {
        App {}
    }

    pub fn show_color_tty(&self, color: Color) {
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

    pub fn show_color(&self, color: Color) {
        if atty::is(Stream::Stdout) {
            self.show_color_tty(color);
        } else {
            let rgba = color.to_rgba();
            println!("#{:02x}{:02x}{:02x}", rgba.r, rgba.g, rgba.b);
        }
    }

    pub fn show_spectrum(&self) {
        const PADDING: usize = 3;
        const WIDTH: usize = 40;

        let mut canvas = Canvas::new(WIDTH + 2 * PADDING, WIDTH + 2 * PADDING);
        canvas.draw_rect(
            PADDING - 1,
            PADDING - 1,
            WIDTH + 2,
            WIDTH + 2,
            TermColor::RGB(100, 100, 100),
        );

        for y in 0..WIDTH {
            for x in 0..WIDTH {
                let rx = (x as f64) / (WIDTH as f64);
                let ry = (y as f64) / (WIDTH as f64);

                let h = 360.0 * rx;
                let s = 0.6;
                let l = 0.81 * ry + 0.05;

                // Start with HSL
                let color = Color::from_hsl(h, s, l);

                // But (slightly) normalize the luminance
                let mut lch = color.to_lch();
                lch.l = (lch.l + ry * 100.0) / 2.0;
                let color = Color::from_lch(lch.l, lch.c, lch.h);

                canvas.draw_rect(PADDING + y, PADDING + x, 1, 1, to_termcolor(&color));
            }
        }

        canvas.print();
    }

    pub fn show_color_list(&self, sort_order: &str) {
        let mut colors: Vec<&NamedColor> = X11_COLORS.iter().map(|r| r).collect();
        if sort_order == "brightness" {
            colors.sort_by_key(|nc| (-nc.color.brightness() * 1000.0) as i32);
        } else if sort_order == "luminance" {
            colors.sort_by_key(|nc| (-nc.color.luminance() * 1000.0) as i32);
        } else if sort_order == "hue" {
            colors.sort_by_key(|nc| (nc.color.to_lch().h * 1000.0) as i32);
        } else if sort_order == "chroma" {
            colors.sort_by_key(|nc| (nc.color.to_lch().c * 1000.0) as i32);
        }
        colors.dedup_by(|n1, n2| n1.color == n2.color);

        for nc in colors {
            let bg = &nc.color;
            let fg = bg.text_color();
            println!(
                "{}",
                to_termcolor(&fg)
                    .on(to_termcolor(&bg))
                    .paint(format!(" {:24}", nc.name))
            );
        }
    }
}
