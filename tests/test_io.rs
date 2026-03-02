use std::{env, fs, process::Command};

#[path = "common/temp_home.rs"]
mod temp_home;

use temp_home::temp_home_dir;

#[test]
fn rcalc_clear_removes_state_file() {
    let home = temp_home_dir("clear");
    let state_dir = home.join(".config").join("rcalc");
    let state_file = state_dir.join("state.json");

    fs::create_dir_all(&state_dir).expect("state directory should be creatable");
    fs::write(
        &state_file,
        r#"{"history":[],"variables":{},"plot_data":null}"#,
    )
    .expect("state file should be writable before clear");

    let status = Command::new(env!("CARGO_BIN_EXE_rcalc"))
        .arg("clear")
        .env("HOME", &home)
        .status()
        .expect("should execute rcalc clear");

    assert!(status.success(), "`rcalc clear` should exit successfully");
    assert!(
        !state_file.exists(),
        "state file should be removed by `rcalc clear`"
    );
}

#[test]
fn rcalc_run_command_is_available() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcalc"))
        .args(["run", "--help"])
        .output()
        .expect("should execute `rcalc run --help`");

    assert!(
        output.status.success(),
        "`rcalc run --help` should exit successfully"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("rcalc run"),
        "help output should contain the run command"
    );
}

#[test]
fn rcalc_clear_command_is_available() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcalc"))
        .args(["clear", "--help"])
        .output()
        .expect("should execute `rcalc clear --help`");

    assert!(
        output.status.success(),
        "`rcalc clear --help` should exit successfully"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("rcalc clear"),
        "help output should contain the clear command"
    );
}
