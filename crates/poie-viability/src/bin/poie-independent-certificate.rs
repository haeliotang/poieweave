// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S4 certificate aggregating the independent-model checks.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn parse_counts(text: &str) -> (usize, usize) {
    let mut rows = 0;
    let mut matched = 0;
    for token in text.split_whitespace() {
        if let Some(value) = token.strip_prefix("rows=") {
            rows = value.parse().expect("rows");
        }
        if let Some(value) = token.strip_prefix("matched=") {
            matched = value.parse().expect("matched");
        }
    }
    (rows, matched)
}

fn main() -> ExitCode {
    let root = root();
    let checks = [
        ("energy", "experiments/s4-energy-model/expected.txt"),
        ("pv", "experiments/s4-energy-pv/expected.txt"),
        ("grease", "experiments/s4-grease-model/expected.txt"),
        ("concurrency", "experiments/s4-concurrency-model/expected.txt"),
        ("time", "experiments/s4-time-model/expected.txt"),
        ("approval-schedule", "experiments/s4-approval-schedule-model/expected.txt"),
    ];
    let mut rows = Vec::new();
    for (name, relative) in checks {
        let text = fs::read_to_string(root.join(relative)).expect("expected file");
        let (cases, matched) = parse_counts(&text);
        rows.push(serde_json::json!({
            "name": name,
            "expected": relative,
            "cases": cases,
            "matched": matched,
        }));
    }
    let certificate = serde_json::json!({
        "schema": "poie.independent-certificate/1",
        "checks": rows,
        "claim_scope": "independent narrow models only; not whole-system equivalence",
    });
    println!("{}", serde_json::to_string_pretty(&certificate).expect("certificate JSON"));
    ExitCode::SUCCESS
}
