use ansi_term::Color as TermColor;
use atty::Stream;
use clap::{
    crate_description, crate_name, crate_version, App, AppSettings, Arg, ArgMatches, SubCommand,
};

mod hdcanvas;
mod parser;
mod x11colors;

use std::io::{self, BufRead};

use crate::hdcanvas::Canvas;
use crate::parser::parse_color;

use pastel::Color;

use x11colors::{NamedColor, X11_COLORS};

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

fn to_termcolor(c: &Color) -> TermColor {
    let rgba = c.to_rgba();
    TermColor::RGB(rgba.r, rgba.g, rgba.b)
}

/// Returns a list of named colors, sorted by the perceived distance to the given color
fn similar_colors(color: &Color) -> Vec<&NamedColor> {
    let mut colors: Vec<&NamedColor> = X11_COLORS.iter().map(|r| r).collect();
    colors.sort_by_key(|nc| nc.color.distance(&color) as i32);
    colors.dedup_by(|n1, n2| n1.color == n2.color);
    colors
}

fn show_color_tty(color: Color) {
    let rgba = color.to_rgba();
    let hsla = color.to_hsla();
    let terminal_color = to_termcolor(&color);

    const PADDING: usize = 2;
    const CHECKERBOARD_SIZE: usize = 20;
    const COLOR_PANEL_SIZE: usize = 14;

    const COLOR_PANEL_POSITION: usize = PADDING + (CHECKERBOARD_SIZE - COLOR_PANEL_SIZE) / 2;
    const TEXT_POSITION_X: usize = CHECKERBOARD_SIZE + 2 * PADDING;
    const TEXT_POSITION_Y: usize = PADDING + 2;

    let mut canvas = Canvas::new(2 * PADDING + CHECKERBOARD_SIZE, 80);
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

fn show_color(color: Color) {
    if atty::is(Stream::Stdout) {
        show_color_tty(color);
    } else {
        let rgba = color.to_rgba();
        println!("#{:02x}{:02x}{:02x}", rgba.r, rgba.g, rgba.b);
    }
}

fn show_color_list(sort_order: &str) {
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
                .paint(format!("{:25}", nc.name))
        );
    }
}

fn run() -> Result<ExitCode> {
    let color_arg = Arg::with_name("color")
        .help(
            "Colors can be specified in many different formats, such as #RRGGBB, RRGGBB, \
             #RGB, 'rgb(…, …, …)', 'hsl(…, …, …)' or simply by the name of the color. \
             If the color argument is not specified, the color will be read \
             from standard input.\n\
             Examples:\
             \n  - cyan\
             \n  - salmon\
             \n  - skyblue\
             \n  - '#ff0077'\
             \n  - f07\
             \n  - 'rgb(216, 180, 140)'\
             \n  - 'hsl(128, 100%, 54%)'",
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
        .global_setting(AppSettings::AllowNegativeNumbers)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .max_term_width(100)
        .about(crate_description!())
        .subcommand(
            SubCommand::with_name("show")
                .about("Display information about the given color on the terminal")
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("saturate")
                .long_about(
                    "Increase the saturation of a color by adding a certain amount to the HSL \
                     saturation channel (a number between 0.0 and 1.0). If the amount is negative, \
                     the color will be desaturated."
                )
                .about("Increase color saturation by a specified amount")
                .arg(Arg::with_name("amount").help("amount of saturation to add").required(true))
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("desaturate")
                .about("Decrease color saturation by a specified amount")
                .long_about(
                    "Decrease the saturation of a color by subtracting a certain amount from the \
                     HSL saturation channel (a number between 0.0 and 1.0). If the amount is negative, \
                     the color will be saturated.

                     See also: to-gray"
                )
                .arg(Arg::with_name("amount").help("amount of saturation to subtract").required(true))
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("lighten")
                .long_about(
                    "Lighten a color by adding a certain amount to the HSL lightness channel (a \
                     number between 0.0 and 1.0). If the amount is negative, the color will be \
                     darkened.",
                )
                .about("Lighten color by a specified amount")
                .arg(Arg::with_name("amount").help("amount of lightness to add").required(true))
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("darken")
                .long_about(
                    "Darken a color by subtracting a certain amount from the lightness channel (a \
                     number between 0.0 and 1.0). If the amount is negative, the color will be \
                     lightened."
                )
                .about("Darken color by a specified amount")
                .arg(Arg::with_name("amount").help("amount of lightness to subtract").required(true))
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("rotate")
                .about("Rotate the hue channel by a specified angle")
                .long_about("Rotate the HSL hue channel of a color by the specified angle (in \
                             degrees). A rotation by 180° returns the complementary color. A \
                             rotation by 360° returns to the original color.")
                .arg(Arg::with_name("degrees").help("angle by which to rotate (in degrees)").required(true))
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("complement")
                .about("Get the complementary color (hue rotated by 180°)")
                .long_about("Compute the complementary color by rotating the HSL hue channel by 180°.")
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("to-gray")
                .about("Completely desaturate a color (while preserving luminance)")
                .long_about(
                    "Completely desaturate the given color while preserving the luminance.\n\
                     \n\
                     For a definition of 'luminance', see:\n\n  \
                       https://www.w3.org/TR/2008/REC-WCAG20-20081211/#relativeluminancedef")
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("Print a list of available color names")
                .arg(Arg::with_name("sort").short("s").long("sort").help("Sort order").possible_values(&["name", "brightness", "luminance", "hue", "chroma"]).default_value("hue"))
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
    } else if let Some(matches) = global_matches.subcommand_matches("rotate") {
        let degrees = number_arg(matches, "degrees")?;
        let color = color_arg(matches)?;
        show_color(color.rotate_hue(degrees));
    } else if let Some(matches) = global_matches.subcommand_matches("complement") {
        let color = color_arg(matches)?;
        show_color(color.complementary());
    } else if let Some(matches) = global_matches.subcommand_matches("to-gray") {
        let color = color_arg(matches)?;
        show_color(color.to_gray());
    } else if let Some(matches) = global_matches.subcommand_matches("list") {
        let sort_order = matches.value_of("sort").unwrap();
        show_color_list(sort_order);
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
