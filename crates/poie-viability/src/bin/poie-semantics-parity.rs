// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S5: causality guard between instantaneous and clock-advanced semantics.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{
    simulate_multi_service_longevity_event_driven, simulate_multi_service_longevity_with_limits, BatteryLimits,
    FailureEvent,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    println!("scenario\tinstant_days\tadvanced_days\tverdict");
    let mut failed = false;
    for scenario in [
        (("base-05", 2.0), "replace-sleeve-p1", 5.0),
        (("base-05", 3.0), "replace-sleeve-p1", 5.0),
        (("base-08", 2.0), "replace-sleeve-p1", 5.0),
    ] {
        let ((base, interval), process_id, repair_days) = scenario;
        let mut model = Model::load(&root.join(format!("scenarios/{base}/model.toml"))).unwrap();
        for lot in &mut model.lots {
            if lot.resource == "sleeve-p1" {
                lot.si = 0.0;
            }
            if lot.resource == "grease" {
                lot.si = 0.15;
            }
        }
        if let Some(process) = model.processes.iter_mut().find(|process| process.id == process_id) {
            process.duration_s = Some(repair_days * 86_400.0);
        }
        let events: Vec<FailureEvent> = (1..=15)
            .map(|index| FailureEvent { at_s: index as f64 * interval * 86_400.0, asset: "pump-p1".into() })
            .collect();
        let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
        let services = ["potable-water", "workshop-operability"];
        let instant =
            simulate_multi_service_longevity_with_limits(&model, &services, 30.0 * 86_400.0, &events, battery).unwrap();
        let advanced =
            simulate_multi_service_longevity_event_driven(&model, &services, 30.0 * 86_400.0, &events, battery, 0.0)
                .unwrap();
        let instant_days = instant.survived_s / 86_400.0;
        let advanced_days = advanced.survived_s / 86_400.0;
        let verdict = if advanced_days + 1e-9 >= instant_days { "ok" } else { "violation" };
        if verdict == "violation" {
            failed = true;
        }
        println!("{base}@{interval:.0}d->{repair_days:.0}d\t{instant_days:.0}\t{advanced_days:.0}\t{verdict}");
    }
    println!("parity={}", if failed { "FAILED" } else { "ok" });
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
