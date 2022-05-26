# unreleased

## Features


## Bugfixes


## Changes


## Other


## Packaging



## v0.9.0

## Features

- Added support for transparency / alpha values, see #131 and #162 (@superhawk610)
- Added support for `NO_COLOR` environment variable, see #143 (@djmattyg007)
- Added new color pickers: Gnome/Wayland via gdbus, zenity, yad, and wcolor (@sigmaSd, @pvonmoradi)

## Packaging

- Added shell completion files again, see #166 (@sharkdp)


# v0.8.1

## Features

- Added `From` and `Display` traits for each color struct, see #133 (@bresilla)

## Other

- Updated `lexical-core` dependency to fix a compile error with newer Rust versions

## Packaging

- `pastel` is now available on snapstore, see #130 (@purveshpatel511)


# v0.8.0

## Features

- Added CMYK output format, see #122 and #123 (@aeter)

## Other

- Completely new CI/CD system via GitHub Actions, see #120 (@rivy)

# v0.7.1

## Bugfixes

- Fixed a bug with the new `ansi-*-escapecode` formats, see #116 (@bbkane)

# v0.7.0

## Changes

- **Breaking:** the existing `ansi-8bit` and `ansi-24bit` formats have been changed to
  print out an escaped ANSI sequence that a user can see in the terminal output.
  The previously existing formats are now available as `ansi-8bit-escapecode` and
  `ansi-24bit-escapecode`. See #113 and #111.

## Features

- All CSS color formats are now supported (see #12)
- Added support for multiple color stops for gradients (`pastel gradient red blue yellow`), see #49 (@felipe-fg)
- Added `-f`/`--force-color` flag as an alias for `--mode=24bit`, see #48 (@samueldple)
- Added `--color-picker <cmd>` to allow users to choose the colorpicker, see #96 (@d-dorazio)
- Added input support for CIELAB, see #3/#101 (@MusiKid)
- Added support for `rgb(255 0 119)`, `rgb(100%,0%,46.7%)`, `gray(20%)`, and many more new CSS syntaxes, see #103 (@MusiKid)
- Faster and more flexible color parser, adding even more CSS color formats, see #105 (@halfbro)

## `pastel` library changes

- `distinct_colors` is now available in the `pastel::distinct` module, see #95 (@rivy)

## Bugfixes

- Added support for non-color consoles (Windows 7), see #91 (@rivy)

## Other

- pastel is now available via Nix, see #100 (@davidtwco)

# v0.6.1

## Other

- Enabled builds for arm, aarch64, and i686
- Fixed build on 32bit platforms

# v0.6.0

## Features

- Added colorblindness simulations via `pastel colorblind`, see #80 (@rozbb)
- Added support for pre-determined colors in `pastel distinct`, see #88 (@d-dorazio)
- Added a new `set` subcommand that can be used to set specific properties of a color (`pastel set lightness 0.4`, `pastel set red 0`, etc.), see #43
- Show the color name in `pastel show` or `pastel color` if it is an exact match, for example:
  `pastel color ff00ff` will show "fuchsia", see #81 (@d-dorazio)
- Add KColorChooser as a supported color picker, see #79 (@data-man)
- Add macOS built-in color picker, see #84 (@erydo)
- Added a new 'count' argument for `pastel pick [<count>]`

## Changes

- `pastel distinct` has seen massive speedups, see #83 (@d-dorazio)

## Bugfixes

- Mixing colors in HSL space with black or white will not rotate the hue towards red (hue 0°), see #76

## Other

- Pastel is now available via Homebrew, see README and #70 (@liamdawson)

# v0.5.3

- Added `rgb-float` as a new format (e.g. `pastel random | pastel format rgb-float`).
- `pastel pick` should now work in 24-bit on Windows, see #45
- Fix crash for `pastel distinct N` with N < 2 (show an error message), see #69

# v0.5.2

* Truecolor support for Windows (@lzybkr)
* Re-arranging of colors in `pastel distinct` so as to maximize the minimal distance to the predecessors
* Fixed small numerical approximation problem in the 'similar colors' computation
* Backported to Rust 1.34

# v0.5.1

- Added shell completion files for bash, zsh, fish and PowerShell.

# v0.5.0

- Added `pastel distinct N` command to generate a set of N visually distinct colors

# v0.4.0

Initial public release
