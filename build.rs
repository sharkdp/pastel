use clap_complete::{generate_to, Shell};
use std::fs;

include!("src/cli/colorpicker_tools.rs");
include!("src/cli/cli.rs");

fn main() {
    let var = std::env::var_os("SHELL_COMPLETIONS_DIR").or_else(|| std::env::var_os("OUT_DIR"));
    let outdir = match var {
        None => return,
        Some(outdir) => outdir,
    };
    fs::create_dir_all(&outdir).unwrap();

    let mut cmd = build_cli();

    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
        generate_to(shell, &mut cmd, crate_name!(), &outdir).unwrap();
    }

    println!("cargo:rustc-cfg=pastel_normal_build");
}
