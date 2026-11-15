// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S3: concurrent repair scheduler over 1-3 robots.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::closure::{Plan, Step, StepAction};
use poie_viability::scheduler::{schedule_plans_with_requirement, ResourceRequirement};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn plan(duration_s: f64, process: &str) -> Plan {
    Plan {
        steps: vec![Step { process: process.into(), action: StepAction::Restore("pump-p1".into()), duration_s }],
        total_duration_s: duration_s,
        remaining_inventory: BTreeMap::new(),
    }
}

fn main() -> ExitCode {
    let root = root();
    let mut model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    for process in &mut model.processes {
        if process.id == "replace-sleeve-p1" || process.id == "replace-lathe-nut" {
            process.occupies = vec!["robot-m1".into(), "robot-m2".into(), "robot-m3".into()];
        }
    }
    let plans = vec![
        plan(3.0 * 3600.0, "replace-sleeve-p1"),
        plan(2.0 * 3600.0, "replace-lathe-nut"),
        plan(2.0 * 3600.0, "replace-sleeve-p1"),
    ];
    println!("robots\tmakespan_h\tplan_ends_h");
    for robots in 1..=3usize {
        let resources = (1..=robots).map(|index| format!("robot-m{index}")).collect::<Vec<_>>();
        let requirement = ResourceRequirement::one_of(&resources);
        let report = schedule_plans_with_requirement(&model, &plans, &resources, &requirement).unwrap();
        let ends = report.plan_end_s.iter().map(|end| format!("{:.0}", end / 3600.0)).collect::<Vec<_>>().join(",");
        println!("{robots}\t{:.0}\t{ends}", report.makespan_s / 3600.0);
    }
    ExitCode::SUCCESS
}
