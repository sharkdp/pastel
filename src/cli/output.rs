use std::io::Write;

use crate::config::Config;
use crate::error::Result;
use crate::hdcanvas::Canvas;
use crate::utility::similar_colors;

use pastel::Color;
use pastel::Format;

// #[derive(Debug)]
pub struct Output<'a> {
    pub handle: &'a mut dyn Write,
    colors_shown: usize,
}

impl Output<'_> {
    pub fn new(handle: &mut dyn Write) -> Output {
        Output {
            handle,
            colors_shown: 0,
        }
    }

    pub fn show_color_tty(&mut self, config: &Config, color: &Color) -> Result<()> {
        let checkerboard_size: usize = 16;
        let color_panel_size: usize = 12;

        let checkerboard_position_y: usize = 0;
        let checkerboard_position_x: usize = config.padding;
        let color_panel_position_y: usize =
            checkerboard_position_y + (checkerboard_size - color_panel_size) / 2;
        let color_panel_position_x: usize =
            config.padding + (checkerboard_size - color_panel_size) / 2;
        let text_position_x: usize = checkerboard_size + 2 * config.padding;
        let text_position_y: usize = 0;

        let mut canvas = Canvas::new(checkerboard_size, 60, config.brush);
        canvas.draw_checkerboard(
            checkerboard_position_y,
            checkerboard_position_x,
            checkerboard_size,
            checkerboard_size,
            &Color::graytone(0.94),
            &Color::graytone(0.71),
        );
        canvas.draw_rect(
            color_panel_position_y,
            color_panel_position_x,
            color_panel_size,
            color_panel_size,
            color,
        );

        let mut text_y_offset = 0;
        let similar = similar_colors(color);

        for (i, nc) in similar.iter().enumerate().take(3) {
            if nc.color == *color {
                canvas.draw_text(
                    text_position_y,
                    text_position_x,
                    &format!("Name: {}", nc.name),
                );
                text_y_offset = 2;
                continue;
            }

            canvas.draw_text(text_position_y + 10 + 2 * i, text_position_x + 7, nc.name);
            canvas.draw_rect(
                text_position_y + 10 + 2 * i,
                text_position_x + 1,
                2,
                5,
                &nc.color,
            );
        }

        #[allow(clippy::identity_op)]
        canvas.draw_text(
            text_position_y + 0 + text_y_offset,
            text_position_x,
            &format!("Hex: {}", color.to_rgb_hex_string(true)),
        );
        canvas.draw_text(
            text_position_y + 2 + text_y_offset,
            text_position_x,
            &format!("RGB: {}", color.to_rgb_string(Format::Spaces)),
        );
        canvas.draw_text(
            text_position_y + 4 + text_y_offset,
            text_position_x,
            &format!("HSL: {}", color.to_hsl_string(Format::Spaces)),
        );

        canvas.draw_text(
            text_position_y + 8 + text_y_offset,
            text_position_x,
            "Most similar:",
        );

        canvas.print(self.handle)
    }

    pub fn show_color(&mut self, config: &Config, color: &Color) -> Result<()> {
        if config.interactive_mode {
            if self.colors_shown < 1 {
                writeln!(self.handle)?
            };
            self.show_color_tty(config, color)?;
            writeln!(self.handle)?;
        } else {
            writeln!(self.handle, "{}", color.to_hsl_string(Format::NoSpaces))?;
        }
        self.colors_shown += 1;

        Ok(())
    }
}
