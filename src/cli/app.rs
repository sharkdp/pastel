use ansi_term::Color as TermColor;
use atty::Stream;

use pastel::Color;

use crate::hdcanvas::Canvas;
use crate::termcolor::to_termcolor;
use crate::utility::similar_colors;
use crate::x11colors::{NamedColor, X11_COLORS};

pub struct App {
    padding: usize,
    colorpicker_width: usize,
}

impl App {
    pub fn new() -> App {
        App {
            padding: 2,
            colorpicker_width: 40,
        }
    }

    pub fn show_color_tty(&self, color: Color) {
        let terminal_color = to_termcolor(&color);

        let checkerboard_size: usize = 20;
        let color_panel_size: usize = 14;

        let color_panel_position: usize = self.padding + (checkerboard_size - color_panel_size) / 2;
        let text_position_x: usize = checkerboard_size + 2 * self.padding;
        let text_position_y: usize = self.padding + 2;

        let mut canvas = Canvas::new(2 * self.padding + checkerboard_size, 55);
        canvas.draw_checkerboard(
            self.padding,
            self.padding,
            checkerboard_size,
            checkerboard_size,
            TermColor::RGB(240, 240, 240),
            TermColor::RGB(180, 180, 180),
        );
        canvas.draw_rect(
            color_panel_position,
            color_panel_position,
            color_panel_size,
            color_panel_size,
            terminal_color,
        );

        canvas.draw_text(
            text_position_y + 0,
            text_position_x,
            &format!("Hex: {}", color.to_rgb_hex_string()),
        );
        canvas.draw_text(
            text_position_y + 2,
            text_position_x,
            &format!("RGB: {}", color.to_rgb_string()),
        );
        canvas.draw_text(
            text_position_y + 4,
            text_position_x,
            &format!("HSL: {}", color.to_hsl_string()),
        );

        canvas.draw_text(text_position_y + 8, text_position_x, "Most similar:");
        let similar = similar_colors(&color);
        for (i, nc) in similar.iter().enumerate().take(3) {
            canvas.draw_text(text_position_y + 10 + 2 * i, text_position_x + 7, nc.name);
            canvas.draw_rect(
                text_position_y + 10 + 2 * i,
                text_position_x + 1,
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
            println!("{}", color.to_hsl_string());
        }
    }

    pub fn show_spectrum(&self) {
        let width = self.colorpicker_width;

        let mut canvas = Canvas::new(width + 2 * self.padding, width + 2 * self.padding);
        canvas.draw_rect(
            self.padding,
            self.padding,
            width + 2,
            width + 2,
            TermColor::RGB(100, 100, 100),
        );

        for y in 0..width {
            for x in 0..width {
                let rx = (x as f64) / (width as f64);
                let ry = (y as f64) / (width as f64);

                let h = 360.0 * rx;
                let s = 0.6;
                let l = 0.81 * ry + 0.05;

                // Start with HSL
                let color = Color::from_hsl(h, s, l);

                // But (slightly) normalize the luminance
                let mut lch = color.to_lch();
                lch.l = (lch.l + ry * 100.0) / 2.0;
                let color = Color::from_lch(lch.l, lch.c, lch.h);

                canvas.draw_rect(
                    self.padding + y + 1,
                    self.padding + x + 1,
                    1,
                    1,
                    to_termcolor(&color),
                );
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
