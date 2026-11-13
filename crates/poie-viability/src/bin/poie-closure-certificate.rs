// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Emit a certificate for a single S1 closure result.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_digest::hex_sha256;
use poie_ontology::model::{AssetStatus, Model};
use poie_viability::closure::{analyze, ClosureOutcome};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model_path = root.join("scenarios/base-04/model.toml");
    let model_sha256 = hex_sha256(&fs::read(&model_path).expect("read model"));
    let model = Model::load(&model_path).unwrap_or_else(|error| panic!("load Base-04: {error}"));
    let mut scenario = model.clone();
    if let Some(asset) = scenario.assets.get_mut("lathe-1") {
        asset.status = AssetStatus::Failed;
    }
    let certificate = match analyze(&scenario, "lathe-1", 7200.0) {
        ClosureOutcome::Restorable(plan) => serde_json::json!({
            "schema": "poie.closure-certificate/1",
            "model": "scenarios/base-04/model.toml",
            "model_sha256": model_sha256,
            "target": "lathe-1",
            "horizon_s": 7200.0,
            "outcome": "RESTORABLE",
            "steps": plan.steps,
            "total_duration_s": plan.total_duration_s,
            "claim_scope": "model-level closure only; not a physical repair claim",
        }),
        ClosureOutcome::Blocked(blocked) => {
            eprintln!("unexpected blocked: {:?}", blocked.reasons);
            return ExitCode::FAILURE;
        }
    };
    println!("{}", serde_json::to_string_pretty(&certificate).expect("certificate JSON"));
    ExitCode::SUCCESS
}
