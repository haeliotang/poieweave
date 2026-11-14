// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 certificate for repair-time semantics results.

use std::process::ExitCode;

fn main() -> ExitCode {
    let certificate = serde_json::json!({
        "schema": "poie.time-certificate/1",
        "dense": {
            "instant_times_h": [0.0, 1.0],
            "event_driven_times_h": [3.0, 6.0],
            "repair_s": 10800.0,
        },
        "sparse": [
            {"load_kw": 0.0, "instant_days": 60, "event_driven_days": 60},
            {"load_kw": 2.6, "instant_days": 16, "event_driven_days": 16},
        ],
        "deadline": {
            "repair_days": 3,
            "failure_day": 10,
            "deadline_day": 11,
            "instant_complete": 10,
            "advanced_complete": 13,
        },
        "grease_boundary": {
            "stocks": [0.10, 0.15, 0.20, 0.30, 0.50],
            "instant_days": [4, 6, 8, 8, 8],
            "advanced_days": [10, 16, 21, 21, 21],
        },
        "regeneration": {
            "base05_instant": [40, 60, 80, 90],
            "base05_advanced": [40, 60, 80, 90],
            "base08_instant": [60, 90, 90, 90],
            "base08_advanced": [60, 90, 90, 90],
        },
        "insert_boundary": {
            "stocks": [1, 2, 3, 4, 6],
            "instant_days": [6, 8, 10, 12, 12],
            "advanced_days": [15, 20, 26, 31, 31],
        },
        "claim_scope": "model/simulation time semantics only; no physical timing claim",
    });
    println!("{}", serde_json::to_string_pretty(&certificate).expect("certificate JSON"));
    ExitCode::SUCCESS
}
