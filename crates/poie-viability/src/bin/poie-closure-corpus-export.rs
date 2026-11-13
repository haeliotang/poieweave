// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Export Rust S1 verdicts for a small differential corpus.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{AssetStatus, Model};
use poie_viability::closure::{analyze, ClosureOutcome};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let models = ["base-01", "base-04", "base-05", "base-06", "base-07", "base-08"];
    let mut output = Vec::new();
    for model_name in models {
        let relative = format!("scenarios/{model_name}/model.toml");
        let model = Model::load(&root.join(&relative)).unwrap_or_else(|error| {
            panic!("load {relative}: {error}");
        });
        let mut targets: Vec<_> = model.assets.keys().cloned().collect();
        targets.sort();
        for target in &targets {
            for horizon_s in [12.0 * 3600.0, 24.0 * 3600.0] {
                let mut scenario = model.clone();
                if let Some(asset) = scenario.assets.get_mut(target) {
                    asset.status = AssetStatus::Failed;
                }
                match analyze(&scenario, target, horizon_s) {
                    ClosureOutcome::Restorable(plan) => output.push(serde_json::json!({
                        "model": relative,
                        "target": target,
                        "horizon_s": horizon_s,
                        "rust": "RESTORABLE",
                        "outcome": "RESTORABLE",
                        "steps": plan.steps,
                        "total_duration_s": plan.total_duration_s,
                    })),
                    ClosureOutcome::Blocked(_) => output.push(serde_json::json!({
                        "model": relative,
                        "target": target,
                        "horizon_s": horizon_s,
                        "rust": "BLOCKED",
                        "outcome": "BLOCKED",
                    })),
                }
            }
        }
    }
    println!("{}", serde_json::to_string_pretty(&output).expect("corpus JSON"));
    ExitCode::SUCCESS
}
