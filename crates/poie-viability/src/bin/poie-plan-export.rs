// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Export deterministic S1 plans for an independent plan verifier.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{AssetStatus, Model};
use poie_viability::closure::{analyze, ClosureOutcome};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let cases = [
        ("scenarios/base-04/model.toml", "lathe-1", 7200.0),
        ("scenarios/base-01/model.toml", "pump-p1", 43200.0),
        ("scenarios/base-05/model.toml", "pump-p1", 10800.0),
        ("scenarios/base-05/model.toml", "lathe-1", 7200.0),
    ];
    let mut output = Vec::new();
    for (relative, target, horizon_s) in cases {
        let model = Model::load(&root.join(relative)).unwrap_or_else(|error| {
            panic!("load {relative}: {error}");
        });
        let mut scenario = model.clone();
        if let Some(asset) = scenario.assets.get_mut(target) {
            asset.status = AssetStatus::Failed;
        }
        let outcome = match analyze(&scenario, target, horizon_s) {
            ClosureOutcome::Restorable(plan) => serde_json::json!({
                "model": relative,
                "target": target,
                "horizon_s": horizon_s,
                "outcome": "RESTORABLE",
                "total_duration_s": plan.total_duration_s,
                "steps": plan.steps,
            }),
            ClosureOutcome::Blocked(blocked) => serde_json::json!({
                "model": relative,
                "target": target,
                "horizon_s": horizon_s,
                "outcome": "BLOCKED",
                "reasons": blocked.reasons.iter().map(ToString::to_string).collect::<Vec<_>>(),
            }),
        };
        output.push(outcome);
    }
    println!("{}", serde_json::to_string_pretty(&output).expect("plan JSON"));
    ExitCode::SUCCESS
}
