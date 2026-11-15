// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Test helper: another process leaves a half-written log line and exits.

use std::fs::OpenOptions;
use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: poie-partial-writer <log-path>");
        return ExitCode::FAILURE;
    };
    let mut file = match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("open failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(error) = file.write_all(b"{\"record_id\":\"site-a/1\",\"kind\":") {
        eprintln!("write failed: {error}");
        return ExitCode::FAILURE;
    }
    let _ = file.flush();
    ExitCode::SUCCESS
}
