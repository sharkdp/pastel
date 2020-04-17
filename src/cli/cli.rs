use clap::{crate_description, crate_name, crate_version, App, AppSettings, Arg, SubCommand};

// Only include `colorpicker_tools` for normal builds (not when compiling `build.rs` where
// the module machinery does not work)
#[cfg(pastel_normal_build)]
use crate::colorpicker_tools::COLOR_PICKER_TOOLS;

const SORT_OPTIONS: &[&str] = &["brightness", "luminance", "hue", "chroma", "random"];
const DEFAULT_SORT_ORDER: &str = "hue";

pub fn build_cli() -> App<'static, 'static> {
    let color_arg = Arg::with_name("color")
        .help(
            "Colors can be specified in many different formats, such as #RRGGBB, RRGGBB, \
             #RGB, 'rgb(…, …, …)', 'hsl(…, …, …)', 'gray(…)' or simply by the name of the \
             color. The identifier '-' can be used to read a single color from standard input. \
             Also, the special identifier 'pick' can be used to run an external color picker \
             to choose a color. If no color argument is specified, colors will be read from \
             standard input.\n\
             Examples (all of these specify the same color):\
             \n  - lightslategray\
             \n  - '#778899'\
             \n  - 778899\
             \n  - 789\
             \n  - 'rgb(119, 136, 153)'\
             \n  - '119,136,153'\
             \n  - 'hsl(210, 14.3%, 53.3%)'",
        )
        .required(false)
        .multiple(true);

    let colorspace_arg = Arg::with_name("colorspace")
        .long("colorspace")
        .short("s")
        .value_name("name")
        .help("The colorspace in which to interpolate")
        .possible_values(&["Lab", "LCh", "RGB", "HSL"])
        .case_insensitive(true)
        .default_value("Lab")
        .required(true);

    App::new(crate_name!())
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
                .long_about("Show and display some information about the given color(s).\n\n\
                Example:\n  \
                  pastel color 556270 4ecdc4 c7f484 ff6b6b c44d58")
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
                .long_about("Generate a list of random colors.\n\n\
                Example:\n  \
                  pastel random -n 20 --strategy lch_hue")
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
            SubCommand::with_name("distinct")
                .about("Generate a set of visually distinct colors")
                .long_about("Generate a set of visually distinct colors by maximizing \
                             the perceived color difference between pairs of colors.\n\n\
                             The default parameters for the optimization procedure \
                             (simulated annealing) should work fine for up to 10-20 colors.")
                .arg(
                    Arg::with_name("number")
                        .help("Number of distinct colors in the set")
                        .takes_value(true)
                        .default_value("10")
                        .required(true)
                        .value_name("count"),
                )
                .arg(
                    Arg::with_name("metric")
                        .long("metric")
                        .short("m")
                        .help("Distance metric to compute mutual color distances. The CIEDE2000 is \
                               more accurate, but also much slower.")
                        .takes_value(true)
                        .possible_values(&["CIEDE2000", "CIE76"])
                        .value_name("name")
                        .default_value("CIE76")
                )
                .arg(
                    Arg::with_name("print-minimal-distance")
                        .long("print-minimal-distance")
                        .help("Only show the optimized minimal distance")
                        .hidden(true)
                )
                .arg(
                    Arg::with_name("verbose")
                        .long("verbose")
                        .short("v")
                        .help("Print simulation output to STDERR")
                ).
                arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("sort-by")
                .about("Sort colors by the given property")
                .long_about("Sort a list of colors by the given property.\n\n\
                Example:\n  \
                  pastel random -n 20 | pastel sort-by hue | pastel format hex")
                .alias("sort")
                .arg(
                    Arg::with_name("sort-order")
                        .help("Sort order")
                        .possible_values(SORT_OPTIONS)
                        .default_value(DEFAULT_SORT_ORDER)
                        .required(true),
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
            SubCommand::with_name("pick")
                .about("Interactively pick a color from the screen (pipette)")
                .long_about("Print a spectrum of colors to choose from. This command requires an \
                external color picker tool to be installed.\n\
                \n\
                Supported tools:\n  \
                  - gpick (https://github.com/thezbyg/gpick)\n  \
                  - xcolor (https://github.com/Soft/xcolor)\n  \
                  - grabc (https://www.muquit.com/muquit/software/grabc/grabc.html)\n  \
                  - colorpicker (https://github.com/Jack12816/colorpicker)\n  \
                  - chameleon (https://github.com/seebye/chameleon)\n  \
                  - KColorChooser (https://kde.org/applications/graphics/org.kde.kcolorchooser)\n  \
                  - macOS built-in color picker")
                .arg(
                    Arg::with_name("count")
                        .help("Number of colors to pick")
                        .default_value("1")
                        .required(true)
                )
        )
        .subcommand(
            SubCommand::with_name("format")
                .about("Convert a color to the given format")
                .long_about("Convert the given color(s) to a specific format.\n\n\
                Example:\n  \
                  pastel random -n 20 | pastel format rgb")
                .arg(
                    Arg::with_name("type")
                        .help("Output format type. Note that the 'ansi-*-escapecode' formats print \
                               ansi escape sequences to the terminal that will not be visible \
                               unless something else is printed in addition.")
                        .possible_values(&["rgb", "rgb-float", "hex",
                                           "hsl", "hsl-hue", "hsl-saturation", "hsl-lightness",
                                           "lch", "lch-lightness", "lch-chroma", "lch-hue",
                                           "lab", "lab-a", "lab-b",
                                           "luminance", "brightness",
                                           "ansi-8bit", "ansi-24bit",
                                           "ansi-8bit-escapecode", "ansi-24bit-escapecode",
                                           "cmyk", "name"])
                        .case_insensitive(true)
                        .default_value("hex")
                        .required(true),
                )
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("paint")
                .about("Print colored text using ANSI escape sequences")
                .arg(
                    Arg::with_name("color")
                        .help("The foreground color. Use '-' to read the color from STDIN.")
                        .required(true),
                )
                .arg(
                    Arg::with_name("text")
                        .help("The text to be printed in color. If no argument is given, \
                               the input is read from STDIN.")
                        .multiple(true)
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
            SubCommand::with_name("gradient")
                .about("Generate an interpolating sequence of colors")
                .long_about("Generate a sequence of colors that interpolates between the specified colors.\n\
                            The interpolation is performed in the specified color space.\n\n\
                            Example:\n  \
                              pastel gradient --colorspace=HSL ffffcc fd8d3c\n  \
                              pastel gradient 555ee4 white d84341 -n 15")
                .arg(
                    Arg::with_name("color")
                        .value_name("color")
                        .help("Color stops in the color gradient")
                        .multiple(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("number")
                        .long("number")
                        .short("n")
                        .help("Number of colors to generate")
                        .takes_value(true)
                        .default_value("10")
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
                     Example:\n  \
                       pastel mix --colorspace=RGB red blue")
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
            SubCommand::with_name("colorblind")
                .about("Simulate a color under a certain colorblindness profile")
                .long_about(
                    "Convert the given color to how it would look to a person with protanopia, \
                    deuteranopia, or tritanopia \n\n\
                     Example:\n  \
                       pastel distinct 3 | pastel colorblind deuter")
                .arg(
                    Arg::with_name("type")
                        .help("The type of colorblindness that should be simulated (protanopia, \
                               deuteranopia, tritanopia)")
                        .possible_values(&["prot", "deuter", "trit"])
                        .case_insensitive(true)
                        .required(true),
                )
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("set")
                .about("Set a color property to a specific value")
                .long_about("Set the given property to a specific value\n\
                Example:\n  \
                  pastel random | pastel set luminance 0.9")
                .arg(
                    Arg::with_name("property")
                        .help("The property that should be changed")
                        .possible_values(&["lightness", "hue", "chroma",
                                           "lab-a", "lab-b",
                                           "red", "green", "blue",
                                           "hsl-hue", "hsl-saturation", "hsl-lightness"])
                        .case_insensitive(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("value")
                        .help("The new numerical value of the property")
                        .required(true),
                )
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("saturate")
                .long_about(
                    "Increase the saturation of a color by adding a certain amount to the HSL \
                     saturation channel. If the amount is negative, the color will be desaturated \
                     instead.",
                )
                .about("Increase color saturation by a specified amount")
                .arg(
                    Arg::with_name("amount")
                        .help("Amount of saturation to add (number between 0.0 and 1.0)")
                        .required(true),
                )
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("desaturate")
                .long_about(
                    "Decrease the saturation of a color by subtracting a certain amount from the \
                     HSL saturation channel. If the amount is negative, the color will be saturated \
                     instead.",
                )
                .about("Decrease color saturation by a specified amount")
                .arg(
                    Arg::with_name("amount")
                        .help("Amount of saturation to subtract (number between 0.0 and 1.0)")
                        .required(true),
                )
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("lighten")
                .long_about(
                    "Lighten a color by adding a certain amount to the HSL lightness channel. \
                     If the amount is negative, the color will be darkened.",
                )
                .about("Lighten color by a specified amount")
                .arg(
                    Arg::with_name("amount")
                        .help("Amount of lightness to add (number between 0.0 and 1.0)")
                        .required(true),
                )
                .arg(color_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("darken")
                .long_about(
                    "Darken a color by subtracting a certain amount from the lightness channel. \
                     If the amount is negative, the color will be lightened.",
                )
                .about("Darken color by a specified amount")
                .arg(
                    Arg::with_name("amount")
                        .help("Amount of lightness to subtract (number between 0.0 and 1.0)")
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
                        .help("angle by which to rotate (in degrees, can be negative)")
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
                .long_about("Create a gray tone from a given lightness value.")
                .arg(
                    Arg::with_name("lightness")
                        .help("Lightness of the created gray tone (number between 0.0 and 1.0)")
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
            SubCommand::with_name("textcolor")
                .about("Get a readable text color for the given background color")
                .long_about("Return a readable foreground text color (either black or white) for a \
                            given background color. This can also be used in the opposite way, \
                            i.e. to create a background color for a given text color.")
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
                .default_value(if output_vt100::try_init().is_ok() {"auto"} else {"off"})
                .hide_possible_values(true)
                .hide_default_value(true)
        )
        .arg(
            Arg::with_name("force-color")
                .short("f")
                .long("force-color")
                .help("Alias for --mode=24bit")
        )
        .arg(
            Arg::with_name("color-picker")
                .long("color-picker")
                .takes_value(true)
                .possible_values(&COLOR_PICKER_TOOLS.iter().map(|t| t.command).collect::<Vec<_>>())
                .case_insensitive(true)
                .help("Use a specific tool to pick the colors")
        )
}
