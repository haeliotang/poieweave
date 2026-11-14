// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2: restorer grease cost under a limited grease stock.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let base = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let events = vec![
        FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p1".into() },
        FailureEvent { at_s: 20.0 * 86400.0, asset: "pump-p1".into() },
        FailureEvent { at_s: 30.0 * 86400.0, asset: "pump-p1".into() },
    ];
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("grease_kg\tmode\tsurvived_days\tblocked_by");
    for grease_kg in [0.10, 0.15, 0.20] {
        for (mode, grease_per_repair) in [("cheap", 0.05), ("expensive", 0.20)] {
            let mut model = base.clone();
            let lot = model.lots.iter_mut().find(|lot| lot.id == "lot-grease").unwrap();
            lot.si = grease_kg;
            let process = model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap();
            for input in &mut process.inputs {
                if input.resource == "grease" {
                    input.si = grease_per_repair;
                }
            }
            let report = simulate_multi_service_longevity_with_limits(
                &model,
                &["potable-water", "workshop-operability"],
                30.0 * 86400.0,
                &events,
                battery,
            )
            .unwrap();
            let blocked = report.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
            println!("{grease_kg:.2}\t{mode}\t{:.0}\t{blocked}", report.survived_s / 86400.0);
        }
    }
    ExitCode::SUCCESS
}
