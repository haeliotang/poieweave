// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S5: survival with scheduled concurrent repairs.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_multi_service_longevity_scheduled, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let mut model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let mut second = model.assets.get("robot-m1").unwrap().clone();
    second.id = "robot-m2".into();
    model.assets.insert("robot-m2".into(), second);
    for process in &mut model.processes {
        if process.id == "replace-sleeve-p1" || process.id == "replace-lathe-nut" {
            process.occupies = vec!["robot-m1".into(), "robot-m2".into()];
        }
    }
    let events =
        vec![FailureEvent { at_s: 0.0, asset: "pump-p1".into() }, FailureEvent { at_s: 0.0, asset: "lathe-1".into() }];
    println!("robots\tcompletion_h\thorizon_days\tblocked");
    for robots in 1..=2usize {
        let resources = (1..=robots).map(|index| format!("robot-m{index}")).collect::<Vec<_>>();
        let report = simulate_multi_service_longevity_scheduled(
            &model,
            &["potable-water", "workshop-operability"],
            30.0 * 86_400.0,
            &events,
            &resources,
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
            0.0,
        )
        .unwrap();
        let blocked = report.log.iter().find_map(|entry| entry.blocked_by.clone()).unwrap_or_else(|| "none".into());
        let completion_h = report.log.iter().map(|entry| entry.at_s).fold(0.0_f64, f64::max) / 3600.0;
        println!("{robots}\t{completion_h:.0}\t{:.0}\t{blocked}", report.survived_s / 86_400.0);
    }
    ExitCode::SUCCESS
}
