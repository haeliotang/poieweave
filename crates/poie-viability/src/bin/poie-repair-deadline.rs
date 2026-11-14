// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S3: repair duration versus a post-failure deadline.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::closure::{analyze, ClosureOutcome};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let mut model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let process = model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap();
    process.duration_s = Some(3.0 * 86_400.0);
    model.assets.get_mut("pump-p1").unwrap().status = poie_ontology::model::AssetStatus::Failed;
    let repair_days = match analyze(&model, "pump-p1", 30.0 * 86_400.0) {
        ClosureOutcome::Restorable(plan) => plan.total_duration_s / 86_400.0,
        ClosureOutcome::Blocked(_) => 0.0,
    };
    let failure_day = 10.0;
    let deadline_day = 11.0;
    println!("repair_days\tfailure_day\tdeadline_day\tinstant_complete\tinstant_meets\tclock_advanced_complete\tclock_advanced_meets");
    println!(
        "{repair_days:.0}\t{failure_day:.0}\t{deadline_day:.0}\t{failure_day:.0}\t{}\t{:.0}\t{}",
        failure_day <= deadline_day,
        failure_day + repair_days,
        failure_day + repair_days <= deadline_day
    );
    ExitCode::SUCCESS
}
