use std::io::{self, Write};

use atty::Stream;
use clap::{crate_description, crate_name, crate_version, App, AppSettings, Arg, SubCommand};

mod commands;
mod config;
mod error;
mod hdcanvas;
mod named;
mod parser;
mod utility;

use commands::Command;
use config::Config;
use error::{PastelError, Result};

use pastel::ansi::{self, Brush};
use pastel::Color;

type ExitCode = i32;

const SORT_OPTIONS: &[&'static str] = &["brightness", "luminance", "hue", "chroma", "random"];
const DEFAULT_SORT_ORDER: &'static str = "hue";

fn write_stderr(c: Color, title: &str, message: &str) {
    writeln!(
        io::stderr(),
        "{}: {}",
        Brush::from_environment().paint(format!("[{}]", title), c),
        message
    )
    .ok();
}

fn run() -> Result<ExitCode> {
    let color_arg = Arg::with_name("color")
        .help(
            "Colors can be specified in many different formats, such as #RRGGBB, RRGGBB, \
             #RGB, 'rgb(…, …, …)', 'hsl(…, …, …)', 'gray(…)' or simply by the name of the \
             color. If no color argument is specified, colors will be read from \
             standard input.\n\
             Examples:\
             \n  - cyan\
             \n  - salmon\
             \n  - skyblue\
             \n  - '#ff0077'\
             \n  - f07\
             \n  - 'rgb(216, 180, 140)'\
             \n  - 'hsl(128, 100%, 54%)'",
        )
        .required(false)
        .multiple(true);

    let colorspace_arg = Arg::with_name("colorspace")
        .long("colorspace")
        .short("s")
        .help("The colorspace in which to interpolate")
        .possible_values(&["rgb", "hsl", "lab", "lch"])
        .default_value("rgb")
        .required(true);

    let app = App::new(crate_name!())
        .version(crate_version!())
        .global_setting(AppSettings::ColorAuto)
        .global_setting(AppSettings::ColoredHelp)
        .global_setting(AppSettings::DeriveDisplayOrder)
        .global_setting(AppSettings::UnifiedHelpMessage)
        .global_setting(AppSettings::InferSubcommands)
        .global_setting(AppSettings::VersionlessSubcommands)
        .global_setting(AppSettings::AllowNegativeNumbers)
        .global_setting(AppSettings::DontCollapseArgsInUsage)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .max_term_width(100)
        .about(crate_description!())
        .subcommand(
            SubCommand::with_name("color")
                .alias("take")
                .alias("show")
                .alias("display")
                .about("Display information about the given color")
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("Show a list of available color names")
                .arg(
                    Arg::with_name("sort-order")
                        .short("s")
                        .long("sort")
                        .help("Sort order")
                        .possible_values(SORT_OPTIONS)
                        .default_value(DEFAULT_SORT_ORDER),
                ),
        )
        .subcommand(
            SubCommand::with_name("random")
                .about("Generate a list of random colors")
                .arg(
                    Arg::with_name("strategy")
                        .long("strategy")
                        .short("s")
                        .help(
                            "Randomization strategy:\n   \
                             vivid:    random hue, limited saturation and lightness values\n   \
                             rgb:      samples uniformly in RGB space\n   \
                             gray:     random gray tone (uniform)\n   \
                             lch_hue:  random hue, fixed lightness and chroma\n\
                             \n\
                             Default strategy: 'vivid'\n ",
                        )
                        .possible_values(&["vivid", "rgb", "gray", "lch_hue"])
                        .hide_default_value(true)
                        .hide_possible_values(true)
                        .default_value("vivid"),
                )
                .arg(
                    Arg::with_name("number")
                        .long("number")
                        .short("n")
                        .help("Number of colors to generate")
                        .takes_value(true)
                        .default_value("10")
                        .value_name("count"),
                ),
        )
        .subcommand(
            SubCommand::with_name("sort-by")
                .about("Sort colors by the given property")
                .alias("sort")
                .arg(
                    Arg::with_name("sort-order")
                        .help("Sort order")
                        .possible_values(SORT_OPTIONS)
                        .default_value(DEFAULT_SORT_ORDER),
                )
                .arg(
                    Arg::with_name("reverse")
                        .long("reverse")
                        .short("r")
                        .help("Reverse the sort order"),
                )
                .arg(
                    Arg::with_name("unique")
                        .long("unique")
                        .short("u")
                        .help("Remove duplicate colors (equality is determined via RGB values)"),
                )
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("paint")
                .about("Print colored text using ANSI escape sequences")
                .arg(
                    Arg::with_name("color")
                        .help("The foreground color. Use '-' to read the color from STDIN")
                        .required(true),
                )
                .arg(
                    Arg::with_name("text")
                        .help("The text to be printed in color")
                        .required(true),
                )
                .arg(
                    Arg::with_name("on")
                        .short("o")
                        .long("on")
                        .help("Use the specified background color")
                        .takes_value(true)
                        .value_name("bg-color"),
                )
                .arg(
                    Arg::with_name("bold")
                        .short("b")
                        .long("bold")
                        .help("Print the text in bold face"),
                )
                .arg(
                    Arg::with_name("italic")
                        .short("i")
                        .long("italic")
                        .help("Print the text in italic font"),
                )
                .arg(
                    Arg::with_name("underline")
                        .short("u")
                        .long("underline")
                        .help("Draw a line below the text"),
                )
                .arg(
                    Arg::with_name("no-newline")
                        .short("n")
                        .long("no-newline")
                        .help("Do not print a trailing newline character"),
                ),
        )
        .subcommand(
            SubCommand::with_name("format")
                .about("Convert a color to the given format")
                .arg(
                    Arg::with_name("type")
                        .help("Output format type")
                        .possible_values(&["rgb", "hex",
                                           "hsl", "hsl-hue", "hsl-saturation", "hsl-lightness",
                                           "lch", "lch-lightness", "lch-chroma", "lch-hue",
                                           "lab", "lab-a", "lab-b",
                                           "luminance", "brightness",
                                           "name"])
                        .required(true),
                )
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("scale")
                .about("Generate a list of colors")
                .arg(
                    Arg::with_name("color-start")
                        .value_name("start")
                        .help("The first color in the color scale")
                        .required(true),
                )
                .arg(
                    Arg::with_name("color-stop")
                        .value_name("stop")
                        .help("The last color in the color scale")
                        .required(true),
                )
                .arg(
                    Arg::with_name("number")
                        .long("number")
                        .short("n")
                        .help("Number of colors to generate")
                        .takes_value(true)
                        .default_value("20")
                        .value_name("count"),
                )
                .arg(
                    colorspace_arg.clone()
                )
        )
        .subcommand(
            SubCommand::with_name("mix")
                .about("Mix two colors in the given colorspace")
                .long_about(
                    "Create new colors by interpolating between two colors in the given colorspace.\n\n\
                     Example:\n\n  \
                       pastel mix --colorspace=lab red blue
                    ")
                .arg(
                    colorspace_arg.clone()
                )
                .arg(
                    Arg::with_name("fraction")
                        .long("fraction")
                        .short("f")
                        .help("The number between 0.0 and 1.0 determining how much to \
                              mix in from the base color.")
                        .required(true)
                        .takes_value(true)
                        .default_value("0.5"),
                )
                .arg(
                    Arg::with_name("base")
                        .value_name("color")
                        .help("The base color which will be mixed with the other colors")
                        .required(true),
                )
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("pick")
                .about("Print a spectrum of colors to choose from")
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("saturate")
                .long_about(
                    "Increase the saturation of a color by adding a certain amount to the HSL \
                     saturation channel (a number between 0.0 and 1.0). If the amount is negative, \
                     the color will be desaturated instead.",
                )
                .about("Increase color saturation by a specified amount")
                .arg(
                    Arg::with_name("amount")
                        .help("amount of saturation to add")
                        .required(true),
                )
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("desaturate")
                .long_about(
                    "Decrease the saturation of a color by subtracting a certain amount from the \
                     HSL saturation channel (a number between 0.0 and 1.0). If the amount is \
                     negative, the color will be saturated instead.",
                )
                .about("Decrease color saturation by a specified amount")
                .arg(
                    Arg::with_name("amount")
                        .help("amount of saturation to subtract")
                        .required(true),
                )
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
                .arg(
                    Arg::with_name("amount")
                        .help("amount of lightness to add")
                        .required(true),
                )
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("darken")
                .long_about(
                    "Darken a color by subtracting a certain amount from the lightness channel (a \
                     number between 0.0 and 1.0). If the amount is negative, the color will be \
                     lightened.",
                )
                .about("Darken color by a specified amount")
                .arg(
                    Arg::with_name("amount")
                        .help("amount of lightness to subtract")
                        .required(true),
                )
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("rotate")
                .about("Rotate the hue channel by the specified angle")
                .long_about(
                    "Rotate the HSL hue channel of a color by the specified angle (in \
                     degrees). A rotation by 180° returns the complementary color. A \
                     rotation by 360° returns to the original color.",
                )
                .arg(
                    Arg::with_name("degrees")
                        .help("angle by which to rotate (in degrees)")
                        .required(true),
                )
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("complement")
                .about("Get the complementary color (hue rotated by 180°)")
                .long_about(
                    "Compute the complementary color by rotating the HSL hue channel by 180°.",
                )
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("gray")
                .about("Create a gray tone from a given lightness")
                .long_about("Create a gray tone from a given lightness value between 0.0 and 1.0.")
                .arg(
                    Arg::with_name("lightness")
                        .help("Lightness of the created gray tone")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("to-gray")
                .about("Completely desaturate a color (preserving luminance)")
                .long_about(
                    "Completely desaturate the given color while preserving the luminance.\n\
                     \n\
                     For a definition of 'luminance', see:\n\n  \
                     https://www.w3.org/TR/2008/REC-WCAG20-20081211/#relativeluminancedef",
                )
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("colorcheck")
                .about("Check if your terminal emulator supports 24-bit colors.")
                .setting(AppSettings::Hidden),
        )
        .arg(
            Arg::with_name("color-mode")
                .long("color-mode")
                .short("m")
                .value_name("mode")
                .help("Specify the terminal color mode: 24bit, 8bit, off, *auto*")
                .possible_values(&["24bit", "8bit", "off", "auto"])
                .default_value("auto")
                .hide_possible_values(true)
                .hide_default_value(true)
        );

    let global_matches = app.get_matches();

    let interactive_mode = atty::is(Stream::Stdout);

    let color_mode = match global_matches
        .value_of("color-mode")
        .expect("required argument")
    {
        "24bit" => Some(ansi::Mode::TrueColor),
        "8bit" => Some(ansi::Mode::Ansi8Bit),
        "off" => None,
        "auto" => {
            if interactive_mode {
                let env_color_mode = std::env::var("PASTEL_COLOR_MODE").ok();
                match env_color_mode.as_ref().map(|s| s.as_str()) {
                    Some("8bit") => Some(ansi::Mode::Ansi8Bit),
                    Some("24bit") => Some(ansi::Mode::TrueColor),
                    Some("off") => None,
                    Some(value) => {
                        return Err(PastelError::UnknownColorMode(value.into()));
                    }
                    None => {
                        let env_colorterm = std::env::var("COLORTERM").ok();
                        match env_colorterm.as_ref().map(|s| s.as_str()) {
                            Some("truecolor") | Some("24bit") => Some(ansi::Mode::TrueColor),
                            _ => {
                                if global_matches.subcommand_name() != Some("paint")
                                    && global_matches.subcommand_name() != Some("colorcheck")
                                {
                                    write_stderr(Color::yellow(), "pastel warning",
                                    "Your terminal emulator does not appear to support 24-bit colors \
                                    (this means that the COLORTERM environment variable is not set to \
                                    'truecolor' or '24bit'). \
                                    pastel will fall back to 8-bit colors, but the range of colors \
                                    will be severely limited.\n\n\
                                    To fix this, follow these steps:\n  \
                                      1. Run 'pastel colorcheck' to test if your terminal emulator\n     \
                                         does, in fact, support 24-bit colors. If this is the case,\n     \
                                         set 'PASTEL_COLOR_MODE=24bit' to force 24-bit mode and to\n     \
                                         remove this warning. Alternatively, make sure that COLORTERM\n     \
                                         is properly set by your terminal emulator.\n  \
                                      2. If your terminal emulator does not support 24-bit colors, set\n     \
                                         'PASTEL_COLOR_MODE=8bit' to remove this warning or try a\n     \
                                         different terminal emulator.\n\n\
                                    \
                                    For more information, see https://gist.github.com/XVilka/8346728\n");
                                }
                                Some(ansi::Mode::Ansi8Bit)
                            }
                        }
                    }
                }
            } else {
                None
            }
        }
        _ => unreachable!("Unknown --color-mode argument"),
    };

    let config = Config {
        padding: 2,
        colorpicker_width: 40,
        colorcheck_width: 8,
        interactive_mode,
        brush: Brush::from_mode(color_mode),
    };

    if let (subcommand, Some(matches)) = global_matches.subcommand() {
        let command = Command::from_string(subcommand);
        command.execute(matches, &config)?;
    } else {
        unreachable!("Subcommand is required");
    }

    Ok(0)
}

fn main() {
    let result = run();
    match result {
        Err(PastelError::StdoutClosed) => {}
        Err(err) => {
            write_stderr(Color::red(), "pastel error", &err.message());
            std::process::exit(1);
        }
        Ok(exit_code) => {
            std::process::exit(exit_code);
        }
    }
}
