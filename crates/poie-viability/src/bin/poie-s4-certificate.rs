// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Certificate for the S4 differential corpus evidence.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_digest::hex_sha256;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let differential_path = root.join("experiments/s4-differential-corpus/expected.txt");
    let plans_path = root.join("experiments/s4-differential-corpus/expected-plans.txt");
    let differential = fs::read_to_string(&differential_path).expect("read differential expected");
    let plans = fs::read_to_string(&plans_path).expect("read plans expected");
    let certificate = serde_json::json!({
        "schema": "poie.s4-certificate/1",
        "differential_expected": "experiments/s4-differential-corpus/expected.txt",
        "differential_expected_sha256": hex_sha256(differential.as_bytes()),
        "differential_summary": differential.trim(),
        "plans_expected": "experiments/s4-differential-corpus/expected-plans.txt",
        "plans_expected_sha256": hex_sha256(plans.as_bytes()),
        "plans_summary": plans.trim(),
        "claim_scope": "independent implementation evidence only; not a formal equivalence proof",
    });
    println!("{}", serde_json::to_string_pretty(&certificate).expect("certificate JSON"));
    ExitCode::SUCCESS
}
