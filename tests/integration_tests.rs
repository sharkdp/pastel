use assert_cmd::prelude::*;
use escargot::CargoRun;
use std::process::Command;
use lazy_static::lazy_static;

lazy_static! {
    static ref CARGO_RUN: CargoRun = escargot::CargoBuild::new()
        .bin("pastel")
        .current_release()
        .run()
        .unwrap();
}

fn pastel() -> Command {
    let mut cmd = CARGO_RUN.command();
    cmd.env_remove("PASTEL_COLOR_MODE");
    cmd
}

#[test]
fn color_reads_colors_from_args() {
    pastel()
        .arg("color")
        .arg("red")
        .assert()
        .success()
        .stdout("hsl(0,100.0%,50.0%)\n");

    pastel()
        .arg("color")
        .arg("red")
        .arg("blue")
        .assert()
        .success()
        .stdout("hsl(0,100.0%,50.0%)\nhsl(240,100.0%,50.0%)\n");

    pastel()
        .arg("color")
        .arg("no color")
        .assert()
        .failure();
}

#[test]
fn color_reads_colors_from_stdin() {
    pastel()
        .arg("color")
        .with_stdin()
        .buffer("red\nblue\n")
        .assert()
        .success()
        .stdout("hsl(0,100.0%,50.0%)\nhsl(240,100.0%,50.0%)\n");

    pastel()
        .arg("color")
        .with_stdin()
        .buffer("no color")
        .assert()
        .failure();
}

#[test]
fn format_basic() {
    pastel()
        .arg("format")
        .arg("hex")
        .arg("red")
        .assert()
        .success()
        .stdout("#ff0000\n");

    pastel()
        .arg("format")
        .arg("rgb")
        .arg("red")
        .arg("blue")
        .assert()
        .success()
        .stdout("rgb(255, 0, 0)\nrgb(0, 0, 255)\n");
}

#[test]
fn pipe_into_format_command() {
    let first = pastel()
        .arg("color")
        .arg("red")
        .arg("teal")
        .arg("hotpink")
        .assert()
        .success();


    pastel()
        .arg("format")
        .arg("name")
        .with_stdin()
        .buffer(String::from_utf8(first.get_output().stdout.clone()).unwrap())
        .assert()
        .success()
        .stdout("red\nteal\nhotpink\n");
}

#[test]
fn sort_by_basic() {
    pastel()
        .arg("sort-by")
        .arg("luminance")
        .arg("gray")
        .arg("white")
        .arg("black")
        .assert()
        .success()
        .stdout("hsl(0,0.0%,0.0%)\nhsl(0,0.0%,50.2%)\nhsl(0,0.0%,100.0%)\n");
}
