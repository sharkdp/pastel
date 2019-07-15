use ansi_term::Color as TermColor;
use atty::Stream;
use clap::ArgMatches;

use std::io::{self, BufRead, Write};

use pastel::ansi::AnsiColor;
use pastel::{Color, Format};

use crate::config::Config;
use crate::hdcanvas::Canvas;
use crate::parser::parse_color;
use crate::termcolor::ToTermColor;
use crate::utility::similar_colors;
use crate::x11colors::{NamedColor, X11_COLORS};
use crate::PastelError;

fn number_arg(matches: &ArgMatches, name: &str) -> Result<f64> {
    let value_str = matches.value_of(name).unwrap();
    value_str
        .parse::<f64>()
        .map_err(|_| PastelError::CouldNotParseNumber(value_str.into()))
}

fn color_from_stdin() -> Result<Color> {
    // TODO: remove duplication between color_from_stdin and colors_from_stdin
    let stdin = io::stdin();
    let mut lock = stdin.lock();

    let mut line = String::new();
    let size = lock
        .read_line(&mut line)
        .map_err(|_| PastelError::ColorInvalidUTF8)?;

    if size == 0 {
        return Err(PastelError::CouldNotReadFromStdin);
    }

    parse_color(&line).ok_or(PastelError::ColorParseError(line.clone()))
}

fn colors_from_stdin() -> Result<Vec<Color>> {
    let stdin = io::stdin();
    let lock = stdin.lock();

    let colors = lock
        .lines()
        .map(|line| {
            let line = line.map_err(|_| PastelError::ColorInvalidUTF8)?;
            parse_color(&line).ok_or(PastelError::ColorParseError(line.clone()))
        })
        .collect::<Result<Vec<_>>>()?;

    if colors.is_empty() {
        return Err(PastelError::CouldNotReadFromStdin);
    }

    Ok(colors)
}

fn color_args(matches: &ArgMatches) -> Result<Vec<Color>> {
    if let Some(color_args) = matches.values_of("color") {
        color_args
            .map(|c| {
                if c == "-" {
                    color_from_stdin()
                } else {
                    parse_color(c).ok_or(PastelError::ColorParseError(c.into()))
                }
            })
            .collect()
    } else {
        if atty::is(Stream::Stdin) {
            return Err(PastelError::ColorArgRequired);
        }

        colors_from_stdin()
    }
}

use crate::Result;

pub trait GenericCommand {
    fn run(&self, matches: &ArgMatches, config: &Config) -> Result<()>;
}

pub trait ColorCommand {
    fn run(&self, matches: &ArgMatches, config: &Config, color: &Color) -> Result<()>;
}

pub fn show_color_tty(config: &Config, color: &Color) {
    let terminal_color = color.to_termcolor();

    let checkerboard_size: usize = 20;
    let color_panel_size: usize = 14;

    let color_panel_position: usize = config.padding + (checkerboard_size - color_panel_size) / 2;
    let text_position_x: usize = checkerboard_size + 2 * config.padding;
    let text_position_y: usize = config.padding + 2;

    let mut canvas = Canvas::new(2 * config.padding + checkerboard_size, 55);
    canvas.draw_checkerboard(
        config.padding,
        config.padding,
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
        &format!("RGB: {}", color.to_rgb_string(Format::Spaces)),
    );
    canvas.draw_text(
        text_position_y + 4,
        text_position_x,
        &format!("HSL: {}", color.to_hsl_string(Format::Spaces)),
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
            nc.color.to_termcolor(),
        );
    }

    canvas.print();
}

pub fn show_color(config: &Config, color: &Color) -> Result<()> {
    if config.interactive_mode {
        show_color_tty(config, color);
    } else {
        println!("{}", color.to_hsl_string(Format::NoSpaces));
    }

    Ok(())
}

struct ShowCommand;

impl ColorCommand for ShowCommand {
    fn run(&self, _: &ArgMatches, config: &Config, color: &Color) -> Result<()> {
        show_color(config, color)
    }
}

struct PickCommand;

impl GenericCommand for PickCommand {
    fn run(&self, _: &ArgMatches, config: &Config) -> Result<()> {
        let width = config.colorpicker_width;

        let mut canvas = Canvas::new(width + 2 * config.padding, width + 2 * config.padding);
        canvas.draw_rect(
            config.padding,
            config.padding,
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
                    config.padding + y + 1,
                    config.padding + x + 1,
                    1,
                    1,
                    color.to_termcolor(),
                );
            }
        }

        canvas.print();

        Ok(())
    }
}

macro_rules! color_command {
    ($cmd_name:ident, $matches:ident, $color:ident, $body:block) => {
        struct $cmd_name;

        impl ColorCommand for $cmd_name {
            fn run(&self, $matches: &ArgMatches, config: &Config, $color: &Color) -> Result<()> {
                let output = $body;
                show_color(&config, &output)
            }
        }
    };
}

color_command!(SaturateCommand, matches, color, {
    let amount = number_arg(matches, "amount")?;
    color.saturate(amount)
});

color_command!(DesaturateCommand, matches, color, {
    let amount = number_arg(matches, "amount")?;
    color.desaturate(amount)
});

color_command!(LightenCommand, matches, color, {
    let amount = number_arg(matches, "amount")?;
    color.lighten(amount)
});

color_command!(DarkenCommand, matches, color, {
    let amount = number_arg(matches, "amount")?;
    color.darken(amount)
});

color_command!(RotateCommand, matches, color, {
    let degrees = number_arg(matches, "degrees")?;
    color.rotate_hue(degrees)
});

color_command!(ComplementCommand, _matches, color, {
    color.complementary()
});

color_command!(ToGrayCommand, _matches, color, { color.to_gray() });

struct GrayCommand;

impl GenericCommand for GrayCommand {
    fn run(&self, matches: &ArgMatches, config: &Config) -> Result<()> {
        let lightness = number_arg(matches, "lightness")?;
        let gray = Color::graytone(lightness);
        show_color(&config, &gray)
    }
}

struct ListCommand;

impl GenericCommand for ListCommand {
    fn run(&self, matches: &ArgMatches, config: &Config) -> Result<()> {
        let sort_order = matches.value_of("sort").expect("required argument");

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

        if config.interactive_mode {
            for nc in colors {
                let bg = &nc.color;
                let fg = bg.text_color();
                println!(
                    "{}",
                    fg.to_termcolor()
                        .on(bg.to_termcolor())
                        .paint(format!(" {:24}", nc.name))
                );
            }
        } else {
            let stdout = io::stdout();
            let mut out = stdout.lock();
            for nc in colors {
                let res = writeln!(out, "{}", nc.name);
                if res.is_err() {
                    break;
                }
            }
        }

        Ok(())
    }
}

struct FormatCommand;

impl ColorCommand for FormatCommand {
    fn run(&self, matches: &ArgMatches, _: &Config, color: &Color) -> Result<()> {
        let format_type = matches.value_of("type").expect("required argument");

        match format_type {
            "rgb" => {
                println!("{}", color.to_rgb_string(Format::Spaces));
            }
            "hsl" => {
                println!("{}", color.to_hsl_string(Format::Spaces));
            }
            "hex" => {
                println!("{}", color.to_rgb_hex_string());
            }
            "ansi-8-bit" => {
                print!("{}", color.to_ansi_sequence_8bit());
            }
            "ansi-24-bit" => {
                print!("{}", color.to_ansi_sequence_24bit());
            }
            &_ => {
                unreachable!("Unknown format type");
            }
        }

        Ok(())
    }
}

struct PaintCommand;

impl GenericCommand for PaintCommand {
    fn run(&self, matches: &ArgMatches, _: &Config) -> Result<()> {
        let text = matches.value_of("text").expect("required argument");

        let fg = matches.value_of("color").expect("required argument");

        if fg.trim() != "default" {
            // TODO: remove duplication - move this into a function and use it in
            // color_args(). Write integration tests
            let fg = if fg == "-" {
                color_from_stdin()?
            } else {
                parse_color(fg).ok_or(PastelError::ColorParseError(fg.into()))?
            };

            let fg_rgba = fg.to_rgba();
            print!(
                "\x1b[38;2;{r};{g};{b}m",
                r = fg_rgba.r,
                g = fg_rgba.g,
                b = fg_rgba.b
            );
        }

        if let Some(bg) = matches.value_of("on") {
            let bg = parse_color(bg).ok_or(PastelError::ColorParseError(bg.into()))?;
            let bg_rgba = bg.to_rgba();
            print!(
                "\x1b[48;2;{r};{g};{b}m",
                r = bg_rgba.r,
                g = bg_rgba.g,
                b = bg_rgba.b
            );
        }

        if matches.is_present("bold") {
            print!("\x1b[1m")
        }

        if matches.is_present("italic") {
            print!("\x1b[3m")
        }

        if matches.is_present("underline") {
            print!("\x1b[4m")
        }

        print!("{}", text);
        print!("\x1b[0m");

        if !matches.is_present("no-newline") {
            println!();
        }

        Ok(())
    }
}

pub enum Command {
    WithColor(Box<dyn ColorCommand>),
    Generic(Box<dyn GenericCommand>),
}

impl Command {
    pub fn from_string(command: &str) -> Command {
        match command {
            "show" => Command::WithColor(Box::new(ShowCommand)),
            "pick" => Command::Generic(Box::new(PickCommand)),
            "saturate" => Command::WithColor(Box::new(SaturateCommand)),
            "desaturate" => Command::WithColor(Box::new(DesaturateCommand)),
            "lighten" => Command::WithColor(Box::new(LightenCommand)),
            "darken" => Command::WithColor(Box::new(DarkenCommand)),
            "rotate" => Command::WithColor(Box::new(RotateCommand)),
            "complement" => Command::WithColor(Box::new(ComplementCommand)),
            "gray" => Command::Generic(Box::new(GrayCommand)),
            "to-gray" => Command::WithColor(Box::new(ToGrayCommand)),
            "list" => Command::Generic(Box::new(ListCommand)),
            "paint" => Command::Generic(Box::new(PaintCommand)),
            "format" => Command::WithColor(Box::new(FormatCommand)),
            _ => unreachable!("Unknown subcommand"),
        }
    }

    pub fn execute(&self, matches: &ArgMatches, config: &Config) -> Result<()> {
        match self {
            Command::Generic(cmd) => cmd.run(matches, config),
            Command::WithColor(cmd) => {
                for color in color_args(matches)? {
                    cmd.run(matches, config, &color)?;
                }

                Ok(())
            }
        }
    }
}
