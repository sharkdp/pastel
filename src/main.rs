#[macro_use]
extern crate clap;

use ansi_term::Colour;
use clap::{App as ClapApp, AppSettings, Arg, SubCommand};
use palette::Srgb;

mod canvas;
mod parser;
mod x11colors;

use crate::canvas::Canvas;
use crate::parser::parse_color;

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

type Color = Srgb<u8>;

fn show_color(color: Color) {
    let terminal_color = Colour::RGB(color.red, color.green, color.blue);

    const RECT_HEIGHT: usize = 8;
    const RECT_WIDTH: usize = 16;
    const RECT_PADDING_X: usize = 4;
    const RECT_PADDING_Y: usize = 2;

    const TEXT_POSITION_X: usize = 2 * RECT_PADDING_X + RECT_WIDTH;

    let mut canvas = Canvas::new(12, 64);
    canvas.draw_rect(
        RECT_PADDING_Y,
        RECT_PADDING_X,
        RECT_HEIGHT,
        RECT_WIDTH,
        terminal_color,
    );
    canvas.draw_text(
        RECT_PADDING_Y + 1,
        TEXT_POSITION_X,
        &format!(
            "Hex: #{:02x}{:02x}{:02x}",
            color.red, color.green, color.blue
        ),
    );
    canvas.draw_text(
        RECT_PADDING_Y + 2,
        TEXT_POSITION_X,
        &format!("RGB: rgb({}, {}, {})", color.red, color.green, color.blue),
    );
    canvas.print();
}

fn run() -> Result<ExitCode> {
    let app = ClapApp::new(crate_name!())
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
