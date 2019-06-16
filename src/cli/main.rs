use ansi_term::Color as TermColor;
use atty::Stream;
use clap::{
    crate_description, crate_name, crate_version, App, AppSettings, Arg, ArgMatches, SubCommand,
};

mod canvas;
mod parser;
mod x11colors;

use std::io::{self, BufRead};

use crate::canvas::Canvas;
use crate::parser::parse_color;

extern crate pastel;
use pastel::Color;

#[derive(Debug, PartialEq)]
enum PastelError {
    ColorParseError,
    CouldNotReadFromStdin,
    ColorArgRequired,
    CouldNotParseNumber,
}

impl PastelError {
    fn message(&self) -> &str {
        match self {
            PastelError::ColorParseError => "could not parse color",
            PastelError::CouldNotReadFromStdin => "could not read color from standard input",
            PastelError::ColorArgRequired => {
                "A color argument needs to be provided on the command line or via a pipe"
            }
            PastelError::CouldNotParseNumber => "Could not parse number",
        }
    }
}

type Result<T> = std::result::Result<T, PastelError>;

type ExitCode = i32;

fn show_color_tty(color: Color) {
    let rgba = color.to_rgba();
    let hsla = color.to_hsla();
    let terminal_color = TermColor::RGB(rgba.r, rgba.g, rgba.b);

    const PADDING: usize = 1;
    const CHECKERBOARD_SIZE: usize = 12;
    const COLOR_PANEL_SIZE: usize = 8;

    const COLOR_PANEL_POSITION: usize = PADDING + (CHECKERBOARD_SIZE - COLOR_PANEL_SIZE) / 2;
    const TEXT_POSITION_X: usize = CHECKERBOARD_SIZE + 2 * PADDING;

    let mut canvas = Canvas::new(2 * PADDING + CHECKERBOARD_SIZE, 26);
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

fn show_color(color: Color) {
    if atty::is(Stream::Stdout) {
        show_color_tty(color);
    } else {
        let rgba = color.to_rgba();
        println!("#{:02x}{:02x}{:02x}", rgba.r, rgba.g, rgba.b);
    }
}

fn run() -> Result<ExitCode> {
    let color_arg = Arg::with_name("color")
        .help(
            "Color argument. Can be specified in many different formats, \
             such as RRGGBB, 'rgb(…,…,…)', 'hsl(…,…,…)' or as a color name. \
             If the color argument is not specified, the color will be read \
             from standard input.",
        )
        .required(false);
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
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("saturate")
                .about(
                    "Increase the saturation of a color by adding a certain amount (number between \
                    -1.0 and 1.0) to the saturation channel.",
                )
                .arg(Arg::with_name("amount").help("amount of saturation to add").required(true))
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("desaturate")
                .about("Opposite of 'saturate'.")
                .arg(Arg::with_name("amount").help("amount of saturation to subtract").required(true))
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("lighten")
                .about(
                    "Lighten a color by adding a certain amount (number between -1.0 and 1.0) \
                     to the lightness channel.",
                )
                .arg(Arg::with_name("amount").help("amount of lightness to add").required(true))
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("darken")
                .about("Opposite of 'lighten'.")
                .arg(Arg::with_name("amount").help("amount of lightness to subtract").required(true))
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("complement")
                .about("Get the complementary color (hue rotated by 180°)")
                .arg(color_arg.clone()),
        );

    let global_matches = app.get_matches();

    let color_arg = |matches: &ArgMatches| -> Result<Color> {
        if let Some(color_arg) = matches.value_of("color") {
            Ok(parse_color(color_arg).ok_or(PastelError::ColorParseError)?)
        } else {
            if atty::is(Stream::Stdin) {
                return Err(PastelError::ColorArgRequired);
            }

            let stdin = io::stdin();
            let mut lock = stdin.lock();

            let mut color_str = String::new();
            lock.read_line(&mut color_str)
                .map_err(|_| PastelError::CouldNotReadFromStdin)?;

            Ok(parse_color(&color_str).ok_or(PastelError::ColorParseError)?)
        }
    };

    let number_arg = |matches: &ArgMatches, name: &str| -> Result<f64> {
        let value_str = matches.value_of(name).unwrap();
        value_str
            .parse::<f64>()
            .map_err(|_| PastelError::CouldNotParseNumber)
    };

    if let Some(matches) = global_matches.subcommand_matches("show") {
        let color = color_arg(matches)?;
        show_color(color);
    } else if let Some(matches) = global_matches.subcommand_matches("saturate") {
        let amount = number_arg(matches, "amount")?;
        let color = color_arg(matches)?;
        show_color(color.saturate(amount));
    } else if let Some(matches) = global_matches.subcommand_matches("desaturate") {
        let amount = number_arg(matches, "amount")?;
        let color = color_arg(matches)?;
        show_color(color.desaturate(amount));
    } else if let Some(matches) = global_matches.subcommand_matches("lighten") {
        let amount = number_arg(matches, "amount")?;
        let color = color_arg(matches)?;
        show_color(color.lighten(amount));
    } else if let Some(matches) = global_matches.subcommand_matches("darken") {
        let amount = number_arg(matches, "amount")?;
        let color = color_arg(matches)?;
        show_color(color.darken(amount));
    } else if let Some(matches) = global_matches.subcommand_matches("complement") {
        let color = color_arg(matches)?;
        show_color(color.complementary());
    } else {
        unreachable!("Unknown subcommand");
    }

    Ok(0)
}

fn main() {
    let result = run();
    match result {
        Err(err) => {
            eprintln!(
                "{}: {}",
                TermColor::Red.paint("[pastel error]"),
                err.message()
            );
            std::process::exit(1);
        }
        Ok(exit_code) => {
            std::process::exit(exit_code);
        }
    }
}
