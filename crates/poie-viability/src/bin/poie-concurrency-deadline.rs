// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S3: do shared-robot repairs miss a common deadline that separate robots meet?

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::closure::{analyze, ClosureOutcome, Plan};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn plan_for(model: &Model, asset: &str) -> Option<Plan> {
    match analyze(model, asset, 24.0 * 3600.0) {
        ClosureOutcome::Restorable(plan) => Some(plan),
        ClosureOutcome::Blocked(_) => None,
    }
}

fn robot_intervals(model: &Model, plan: &Plan, robot: &str) -> Vec<(f64, f64)> {
    let mut now = 0.0;
    let mut intervals = Vec::new();
    for step in &plan.steps {
        let process = model.processes.iter().find(|process| process.id == step.process).unwrap();
        if process.occupies.iter().any(|asset| asset == robot) {
            intervals.push((now, now + step.duration_s));
        }
        now += step.duration_s;
    }
    intervals
}

fn overlaps(left: &[(f64, f64)], right: &[(f64, f64)]) -> bool {
    left.iter().any(|(left_start, left_end)| {
        right.iter().any(|(right_start, right_end)| left_start < right_end && right_start < left_end)
    })
}

fn main() -> ExitCode {
    let root = root();
    let base = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    println!("layout\tpump_s\tlathe_s\tdeadline_s\tsequential_meets\tparallel_meets");
    for (layout, lathe_robot) in [("one-robot", "robot-m1"), ("two-robots", "robot-m2")] {
        let mut model = base.clone();
        for process in &mut model.processes {
            if process.id == "replace-sleeve-p1" {
                process.occupies = vec!["robot-m1".into()];
            }
            if process.id == "replace-lathe-nut" {
                process.occupies = vec![lathe_robot.into()];
            }
        }
        for (id, asset) in [("pump-p1", "pump-p1"), ("lathe-1", "lathe-1")] {
            if let Some(item) = model.assets.get_mut(asset) {
                item.status = poie_ontology::model::AssetStatus::Failed;
            }
            let _ = id;
        }
        model.assets.entry("robot-m2".into()).or_insert_with(|| {
            let mut asset = base.assets["robot-m1"].clone();
            asset.id = "robot-m2".into();
            asset
        });
        let pump = plan_for(&model, "pump-p1").unwrap();
        let lathe = plan_for(&model, "lathe-1").unwrap();
        let pump_robot = robot_intervals(&model, &pump, "robot-m1");
        let lathe_robot_intervals = robot_intervals(&model, &lathe, lathe_robot);
        let shared = lathe_robot == "robot-m1" && overlaps(&pump_robot, &lathe_robot_intervals);
        let deadline_s = 4.0 * 3600.0;
        let sequential = pump.total_duration_s + lathe.total_duration_s;
        let parallel = if shared { sequential } else { pump.total_duration_s.max(lathe.total_duration_s) };
        println!(
            "{layout}\t{:.0}\t{:.0}\t{:.0}\t{}\t{}",
            pump.total_duration_s,
            lathe.total_duration_s,
            deadline_s,
            sequential <= deadline_s,
            parallel <= deadline_s
        );
    }
    ExitCode::SUCCESS
}
