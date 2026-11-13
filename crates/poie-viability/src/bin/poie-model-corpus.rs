// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Export all-asset verdicts for an arbitrary model path (differential mutation corpus).

use std::path::Path;
use std::process::ExitCode;

use poie_ontology::model::{AssetStatus, Model};
use poie_viability::closure::{analyze, ClosureOutcome};

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: poie-model-corpus <model.toml>");
        return ExitCode::from(2);
    };
    let model = Model::load(Path::new(&path)).unwrap_or_else(|error| {
        panic!("load {path}: {error}");
    });
    let mut targets: Vec<_> = model.assets.keys().cloned().collect();
    targets.sort();
    let mut output = Vec::new();
    for target in targets {
        let mut scenario = model.clone();
        if let Some(asset) = scenario.assets.get_mut(&target) {
            asset.status = AssetStatus::Failed;
        }
        let outcome = match analyze(&scenario, &target, 24.0 * 3600.0) {
            ClosureOutcome::Restorable(_) => "RESTORABLE",
            ClosureOutcome::Blocked(_) => "BLOCKED",
        };
        output.push(serde_json::json!({ "target": target, "outcome": outcome }));
    }
    println!("{}", serde_json::to_string_pretty(&output).expect("model corpus JSON"));
    ExitCode::SUCCESS
}
