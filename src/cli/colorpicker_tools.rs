use lazy_static::lazy_static;

pub struct ColorPickerTool {
    pub command: &'static str,
    pub args: &'static [&'static str],
    pub version_args: &'static [&'static str],
    pub version_output_starts_with: &'static [u8],
}

lazy_static! {
    pub static ref COLOR_PICKER_TOOLS: Vec<ColorPickerTool> = vec![
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
        },
        ColorPickerTool {
            command: "gpick",
            args: &["--pick", "--single", "--output"],
            version_args: &["--version"],
            version_output_starts_with: b"Gpick",
        },
        ColorPickerTool {
            command: "xcolor",
            args: &["--format", "hex"],
            version_args: &["--version"],
            version_output_starts_with: b"xcolor",
        },
        ColorPickerTool {
            command: "grabc",
            args: &["-hex"],
            version_args: &["-v"],
            version_output_starts_with: b"grabc",
        },
        ColorPickerTool {
            command: "colorpicker",
            args: &["--one-shot", "--short"],
            version_args: &["--help"],
            version_output_starts_with: b"",
        },
        ColorPickerTool {
            command: "chameleon",
            args: &[],
            version_args: &["-h"],
            version_output_starts_with: b"",
        },
        ColorPickerTool {
            command: "kcolorchooser",
            args: &["--print"],
            version_args: &["-v"],
            version_output_starts_with: b"kcolorchooser",
        },
    ];
}
