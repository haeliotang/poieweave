// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Certificate for the Base-06 independent-vs-joint coverage result.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_digest::hex_sha256;
use poie_ontology::model::Model;
use poie_viability::survival::{joint_coverage, BatteryLimits};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model_path = root.join("scenarios/base-06/model.toml");
    let model_sha256 = hex_sha256(&fs::read(&model_path).expect("read model"));
    let model = Model::load(&model_path).unwrap_or_else(|error| panic!("load Base-06: {error}"));
    let report = joint_coverage(
        &model,
        "potable-water",
        12.0 * 3600.0,
        2,
        Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
    )
    .unwrap();
    let gaps: Vec<_> = report
        .cases
        .iter()
        .filter(|case| case.independent_recoverable && !case.joint_recoverable)
        .map(|case| case.failed.clone())
        .collect();
    let certificate = serde_json::json!({
        "schema": "poie.coverage-certificate/1",
        "model": "scenarios/base-06/model.toml",
        "model_sha256": model_sha256,
        "service": "potable-water",
        "horizon_s": 43200.0,
        "max_failures": 2,
        "total": report.total(),
        "independent_recoverable": report.independent_recoverable,
        "joint_recoverable": report.joint_recoverable,
        "optimistic_gap": report.optimistic_gap(),
        "gap_pairs": gaps,
        "claim_scope": "model-level coverage only; not a joint survival certificate",
    });
    println!("{}", serde_json::to_string_pretty(&certificate).expect("certificate JSON"));
    ExitCode::SUCCESS
}
