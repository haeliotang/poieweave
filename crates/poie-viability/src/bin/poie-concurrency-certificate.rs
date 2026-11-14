// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S3 certificate for concurrency and shared-resource contention.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::closure::{analyze, ClosureOutcome, Plan};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn plan_for(model: &Model, asset: &str) -> Plan {
    match analyze(model, asset, 24.0 * 3600.0) {
        ClosureOutcome::Restorable(plan) => plan,
        ClosureOutcome::Blocked(blocked) => panic!("{asset} blocked: {:?}", blocked.reasons),
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
    let mut rows = Vec::new();
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
        model.assets.entry("robot-m2".into()).or_insert_with(|| {
            let mut asset = base.assets["robot-m1"].clone();
            asset.id = "robot-m2".into();
            asset
        });
        for asset in ["pump-p1", "lathe-1"] {
            if let Some(item) = model.assets.get_mut(asset) {
                item.status = poie_ontology::model::AssetStatus::Failed;
            }
        }
        let pump = plan_for(&model, "pump-p1");
        let lathe = plan_for(&model, "lathe-1");
        let shared = lathe_robot == "robot-m1"
            && overlaps(&robot_intervals(&model, &pump, "robot-m1"), &robot_intervals(&model, &lathe, lathe_robot));
        let deadline_s = 4.0 * 3600.0;
        let sequential_s = pump.total_duration_s + lathe.total_duration_s;
        let parallel_s = if shared { sequential_s } else { pump.total_duration_s.max(lathe.total_duration_s) };
        rows.push(serde_json::json!({
            "layout": layout,
            "pump_s": pump.total_duration_s,
            "lathe_s": lathe.total_duration_s,
            "occupancy_overlap": shared,
            "sequential_s": sequential_s,
            "parallel_s": parallel_s,
            "deadline_s": deadline_s,
            "sequential_meets": sequential_s <= deadline_s,
            "parallel_meets": parallel_s <= deadline_s,
        }));
    }
    let certificate = serde_json::json!({
        "schema": "poie.concurrency-certificate/1",
        "rows": rows,
        "claim_scope": "model/simulation scheduling observation only; no hardware authority claim",
    });
    println!("{}", serde_json::to_string_pretty(&certificate).expect("certificate JSON"));
    ExitCode::SUCCESS
}
