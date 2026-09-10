use std::io::{self, Write};
use std::process::ExitCode;

use tauri_remote_app_cli::{run_cli, UnwiredAdapter};

fn main() -> ExitCode {
    // Replace `UnwiredAdapter` with the application's reviewed adapter to its
    // running Rust authority. Do not instantiate a second state owner here.
    let mut adapter = UnwiredAdapter;
    let run = run_cli(std::env::args_os().skip(1), &mut adapter);

    if !run.stdout.is_empty() {
        let _ = io::stdout().write_all(run.stdout.as_bytes());
    }
    if !run.stderr.is_empty() {
        let _ = io::stderr().write_all(run.stderr.as_bytes());
    }
    ExitCode::from(run.exit_code)
}
