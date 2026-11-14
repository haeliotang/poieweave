// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S1: does closure choose the shortest alternative restorer or the first feasible one?

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::closure::{analyze, ClosureOutcome};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn outcome(model: &Model, horizon_s: f64) -> String {
    match analyze(model, "pump-p1", horizon_s) {
        ClosureOutcome::Restorable(plan) => format!("RESTORABLE {:.0}s", plan.total_duration_s),
        ClosureOutcome::Blocked(blocked) => {
            format!("BLOCKED {}", blocked.reasons.first().map(ToString::to_string).unwrap_or_default())
        }
    }
}

fn main() -> ExitCode {
    let root = root();
    let base = Model::load(&root.join("scenarios/base-05/model.toml")).unwrap();
    let mut slow = base.processes.iter().find(|process| process.id == "replace-sleeve-p1").unwrap().clone();
    slow.id = "replace-sleeve-slow".into();
    slow.duration_s = Some(6.0 * 3600.0);
    let mut model = base.clone();
    model.add_process(slow);
    let slow_index = model.processes.len() - 1;
    println!("order=fast-first\t{}", outcome(&model, 4.0 * 3600.0));
    let restorers = model.restorers.get_mut("pump-p1").unwrap();
    restorers.retain(|index| *index != slow_index);
    restorers.insert(0, slow_index);
    println!("order=slow-first\t{}", outcome(&model, 4.0 * 3600.0));
    ExitCode::SUCCESS
}
