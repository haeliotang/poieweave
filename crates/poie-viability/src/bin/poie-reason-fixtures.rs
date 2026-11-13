// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Export blocked reasons for missing-resource and unverified-process fixtures.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{AssetStatus, Model};
use poie_viability::closure::{analyze, ClosureOutcome};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let cases = [("scenarios/base-09/model.toml", "pump-p1"), ("scenarios/base-10/model.toml", "lathe-1")];
    let mut output = Vec::new();
    for (relative, target) in cases {
        let model = Model::load(&root.join(relative)).unwrap_or_else(|error| {
            panic!("load {relative}: {error}");
        });
        let mut scenario = model.clone();
        if let Some(asset) = scenario.assets.get_mut(target) {
            asset.status = AssetStatus::Failed;
        }
        let reasons = match analyze(&scenario, target, 24.0 * 3600.0) {
            ClosureOutcome::Restorable(_) => vec!["RESTORABLE".to_string()],
            ClosureOutcome::Blocked(blocked) => blocked.reasons.iter().map(ToString::to_string).collect(),
        };
        output.push(serde_json::json!({
            "model": relative,
            "target": target,
            "horizon_s": 24.0 * 3600.0,
            "reasons": reasons,
        }));
    }
    println!("{}", serde_json::to_string_pretty(&output).expect("fixture JSON"));
    ExitCode::SUCCESS
}
