use ansi_term::Color as TermColor;
use clap::{crate_description, crate_name, crate_version, App, AppSettings, Arg, SubCommand};

mod canvas;
mod parser;
mod x11colors;

use crate::canvas::Canvas;
use crate::parser::parse_color;

extern crate pastel;
use pastel::Color;

#[derive(Debug, PartialEq)]
enum PastelError {
    ColorParseError,
}

impl PastelError {
    fn message(&self) -> &str {
        match self {
            PastelError::ColorParseError => "could not parse color",
        }
    }
}

type Result<T> = std::result::Result<T, PastelError>;

type ExitCode = i32;

fn show_color(color: Color) {
    let rgba = color.to_rgba();
    let hsla = color.to_hsla();
    let terminal_color = TermColor::RGB(rgba.r, rgba.g, rgba.b);

    const PADDING: usize = 1;
    const CHECKERBOARD_SIZE: usize = 12;
    const COLOR_PANEL_SIZE: usize = 8;

    const COLOR_PANEL_POSITION: usize = PADDING + (CHECKERBOARD_SIZE - COLOR_PANEL_SIZE) / 2;
    const TEXT_POSITION_X: usize = CHECKERBOARD_SIZE + 2 * PADDING;

    let mut canvas = Canvas::new(2 * PADDING + CHECKERBOARD_SIZE, 40);
    canvas.draw_checkerboard(
        PADDING,
        PADDING,
        CHECKERBOARD_SIZE,
        CHECKERBOARD_SIZE,
        ansi_term::Color::RGB(240, 240, 240),
        ansi_term::Color::RGB(180, 180, 180),
    );
    canvas.draw_rect(
        COLOR_PANEL_POSITION,
        COLOR_PANEL_POSITION,
        COLOR_PANEL_SIZE,
        COLOR_PANEL_SIZE,
        terminal_color,
    );

    canvas.draw_text(
        PADDING + 1,
        TEXT_POSITION_X,
        &format!("Hex: #{:02x}{:02x}{:02x}", rgba.r, rgba.g, rgba.b),
    );
    canvas.draw_text(
        PADDING + 2,
        TEXT_POSITION_X,
        &format!("RGB: rgb({},{},{})", rgba.r, rgba.g, rgba.b),
    );
    canvas.draw_text(
        PADDING + 3,
        TEXT_POSITION_X,
        &format!(
            "HSL: hsl({:.0},{:.0}%,{:.0}%)",
            hsla.h,
            100.0 * hsla.s,
            100.0 * hsla.l
        ),
    );
    canvas.print();
}

fn run() -> Result<ExitCode> {
    let app = App::new(crate_name!())
        .version(crate_version!())
        .global_setting(AppSettings::ColorAuto)
        .global_setting(AppSettings::ColoredHelp)
        .global_setting(AppSettings::DeriveDisplayOrder)
        .global_setting(AppSettings::UnifiedHelpMessage)
        .global_setting(AppSettings::InferSubcommands)
        .global_setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .max_term_width(100)
        .about(crate_description!())
        .subcommand(
            SubCommand::with_name("show")
                .about("Show the given color on the terminal")
                .arg(Arg::with_name("color").help("Color to show").required(true)),
        );

    let global_matches = app.get_matches();

    if let Some(matches) = global_matches.subcommand_matches("show") {
        let color_arg = matches.value_of("color").unwrap();
        let color = parse_color(color_arg).ok_or(PastelError::ColorParseError)?;

        show_color(color);
    } else {
        unreachable!("Unknown subcommand");
    }

    Ok(0)
}

fn main() {
    let result = run();
    match result {
        Err(err) => {
            eprintln!("Error: {}", err.message());
            std::process::exit(1);
        }
        Ok(exit_code) => {
            std::process::exit(exit_code);
        }
    }
}
