use clap::Shell;
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

    let mut app = build_cli();
    app.gen_completions("pastel", Shell::Bash, &outdir);
    app.gen_completions("pastel", Shell::Fish, &outdir);
    app.gen_completions("pastel", Shell::Zsh, &outdir);
    app.gen_completions("pastel", Shell::PowerShell, &outdir);

    println!("cargo:rustc-cfg=pastel_normal_build");
}
