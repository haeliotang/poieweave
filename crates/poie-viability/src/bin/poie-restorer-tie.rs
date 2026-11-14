// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S1: tie-break between equal-duration restorers with different input costs.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::closure::{analyze, ClosureOutcome};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn report(model: &Model) {
    match analyze(model, "pump-p1", 4.0 * 3600.0) {
        ClosureOutcome::Restorable(plan) => {
            let grease = plan.remaining_inventory.get("grease").copied().unwrap_or(0.0);
            println!(
                "steps={:?} duration={:.0}s grease_left={:.2}",
                plan.steps.iter().map(|step| step.process.clone()).collect::<Vec<_>>(),
                plan.total_duration_s,
                grease
            );
        }
        ClosureOutcome::Blocked(blocked) => {
            println!("BLOCKED {}", blocked.reasons.first().map(ToString::to_string).unwrap_or_default());
        }
    }
}

fn main() -> ExitCode {
    let root = root();
    let base = Model::load(&root.join("scenarios/base-05/model.toml")).unwrap();
    let mut expensive = base.processes.iter().find(|process| process.id == "replace-sleeve-p1").unwrap().clone();
    expensive.id = "replace-sleeve-expensive".into();
    for input in &mut expensive.inputs {
        if input.resource == "grease" {
            input.si = 0.20;
        }
    }
    for order in ["cheap-first", "expensive-first"] {
        let mut model = base.clone();
        model.add_process(expensive.clone());
        let expensive_index = model.processes.len() - 1;
        if order == "expensive-first" {
            let restorers = model.restorers.get_mut("pump-p1").unwrap();
            restorers.retain(|index| *index != expensive_index);
            restorers.insert(0, expensive_index);
        }
        print!("{order}\t");
        report(&model);
    }
    ExitCode::SUCCESS
}
