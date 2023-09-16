use once_cell::sync::Lazy;

pub struct ColorPickerTool {
    pub command: &'static str,
    pub args: &'static [&'static str],
    pub version_args: &'static [&'static str],
    pub version_output_starts_with: &'static [u8],
    #[allow(clippy::type_complexity)]
    /// Post-Process the output of the color picker tool
    pub post_process: Option<fn(String) -> Result<String, &'static str>>,
}

pub static COLOR_PICKER_TOOLS: Lazy<Vec<ColorPickerTool>> = Lazy::new(|| {
    vec![
        #[cfg(target_os = "macos")]
        ColorPickerTool {
            command: "osascript",
            // NOTE: This does not use `console.log` to print the value as you might expect,
            // because that gets written to stderr instead of stdout regardless of the `-s o` flag.
            // (This is accurate as of macOS Mojave/10.14.6).
            // See related: https://apple.stackexchange.com/a/278395
            args: &[
                "-l",
                "JavaScript",
                "-s",
                "o",
                "-e",
                "
                const app = Application.currentApplication();\n
                app.includeStandardAdditions = true;\n
                const rgb = app.chooseColor({defaultColor: [0.5, 0.5, 0.5]})\n
                  .map(n => Math.round(n * 255))\n
                  .join(', ');\n
                `rgb(${rgb})`;\n
            ",
            ],
            version_args: &["-l", "JavaScript", "-s", "o", "-e", "'ok';"],
            version_output_starts_with: b"ok",
            post_process: None,
        },
        ColorPickerTool {
            command: "gpick",
            args: &["--pick", "--single", "--output"],
            version_args: &["--version"],
            version_output_starts_with: b"Gpick",
            post_process: None,
        },
        ColorPickerTool {
            command: "xcolor",
            args: &["--format", "hex"],
            version_args: &["--version"],
            version_output_starts_with: b"xcolor",
            post_process: None,
        },
        ColorPickerTool {
            command: "wcolor",
            args: &["--format", "hex"],
            version_args: &["--version"],
            version_output_starts_with: b"wcolor",
            post_process: None,
        },
        ColorPickerTool {
            command: "grabc",
            args: &["-hex"],
            version_args: &["-v"],
            version_output_starts_with: b"grabc",
            post_process: None,
        },
        ColorPickerTool {
            command: "colorpicker",
            args: &["--one-shot", "--short"],
            version_args: &["--help"],
            version_output_starts_with: b"",
            post_process: None,
        },
        ColorPickerTool {
            command: "chameleon",
            args: &[],
            version_args: &["-h"],
            version_output_starts_with: b"",
            post_process: None,
        },
        ColorPickerTool {
            command: "kcolorchooser",
            args: &["--print"],
            version_args: &["-v"],
            version_output_starts_with: b"kcolorchooser",
            post_process: None,
        },
        ColorPickerTool {
            command: "zenity",
            args: &["--color-selection"],
            version_args: &["--version"],
            version_output_starts_with: b"",
            post_process: None,
        },
        ColorPickerTool {
            command: "yad",
            args: &["--color"],
            version_args: &["--version"],
            version_output_starts_with: b"",
            post_process: None,
        },
        ColorPickerTool {
            command: "hyprpicker",
            args: &[],
            version_args: &["-h"],
            version_output_starts_with: b"",
            post_process: None,
        },
        #[cfg(target_os = "linux")]
        ColorPickerTool {
            command: "gdbus",
            args: &[
                "call",
                "--session",
                "--dest",
                "org.gnome.Shell.Screenshot",
                "--object-path",
                "/org/gnome/Shell/Screenshot",
                "--method",
                "org.gnome.Shell.Screenshot.PickColor",
            ],
            version_args: &[
                "introspect",
                "--session",
                "--dest",
                "org.gnome.Shell.Screenshot",
                "--object-path",
                "/org/gnome/Shell/Screenshot",
            ],
            version_output_starts_with: b"node /org/gnome/Shell/Screenshot",
            post_process: Some(gdbus_parse_color),
        },
    ]
});

pub static COLOR_PICKER_TOOL_NAMES: Lazy<Vec<&'static str>> =
    Lazy::new(|| COLOR_PICKER_TOOLS.iter().map(|t| t.command).collect());

#[cfg(target_os = "linux")]
pub fn gdbus_parse_color(raw: String) -> Result<String, &'static str> {
    const PARSE_ERROR: &str = "Unexpected gdbus output format";
    let rgb = raw
        .split('(')
        .nth(2)
        .ok_or(PARSE_ERROR)?
        .split(')')
        .next()
        .ok_or(PARSE_ERROR)?;
    let rgb = rgb
        .split(',')
        .map(|v| v.trim().parse::<f64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| PARSE_ERROR)?;
    if rgb.len() != 3 {
        return Err(PARSE_ERROR);
    }
    Ok(format!(
        "rgb({}%,{}%,{}%)",
        rgb[0] * 100.,
        rgb[1] * 100.,
        rgb[2] * 100.
    ))
}
