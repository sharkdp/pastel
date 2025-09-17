use clap_complete::{generate_to, Shell};
use clap_mangen::Man;
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

    let man = Man::new(cmd.clone());
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer).expect("Man page generation failed");

    let man_path = std::path::Path::new(&outdir).join("pastel.1");
    fs::write(man_path, buffer).expect("Failed to write main man page");

    for subcommand in cmd.get_subcommands() {
        let subcommand_name = subcommand.get_name();

        // help command doesn't need its own man page
        if subcommand_name == "help" {
            continue;
        }

        let man = Man::new(subcommand.clone());
        let mut buffer: Vec<u8> = Default::default();
        man.render(&mut buffer)
            .expect("Subcommand man page generation failed");

        let man_path = std::path::Path::new(&outdir).join(format!("pastel-{}.1", subcommand_name));
        fs::write(man_path, buffer).expect("Failed to write subcommand man page");
    }

    println!("cargo:rustc-cfg=pastel_normal_build");
}
