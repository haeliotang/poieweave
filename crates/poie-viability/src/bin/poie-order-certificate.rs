// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Certificate for the Base-06 repair-order negative result.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_digest::hex_sha256;
use poie_ontology::model::Model;
use poie_viability::survival::{joint_coverage_with_order_search, BatteryLimits};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model_path = root.join("scenarios/base-06/model.toml");
    let model = Model::load(&model_path).unwrap();
    let mut rows = Vec::new();
    for max_failures in [2, 3] {
        let report = joint_coverage_with_order_search(
            &model,
            "potable-water",
            12.0 * 3600.0,
            max_failures,
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        )
        .unwrap();
        rows.push(serde_json::json!({
            "max_failures": max_failures,
            "recoverable_cases": report.total,
            "standard_joint": report.standard_joint,
            "best_joint": report.best_joint,
            "order_sensitive": report.order_sensitive(),
        }));
    }
    let certificate = serde_json::json!({
        "schema": "poie.order-certificate/1",
        "model": "scenarios/base-06/model.toml",
        "model_sha256": hex_sha256(&fs::read(&model_path).unwrap()),
        "rows": rows,
        "conclusion": "repair order does not close the Base-06 shared-material gap",
        "claim_scope": "model-level ordering result only; not a scheduling optimality claim",
    });
    println!("{}", serde_json::to_string_pretty(&certificate).expect("certificate JSON"));
    ExitCode::SUCCESS
}
