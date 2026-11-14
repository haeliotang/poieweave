// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S3 compares instantaneous repairs with repair-duration clock advancement.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{
    simulate_multi_service_longevity_event_driven, simulate_multi_service_longevity_with_load, BatteryLimits,
    FailureEvent,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn event_times(report: &poie_viability::survival::SurvivalReport) -> Vec<f64> {
    report.log.iter().filter(|entry| entry.failed.is_some()).map(|entry| entry.at_s / 3600.0).collect()
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let events = vec![
        FailureEvent { at_s: 0.0, asset: "pump-p1".into() },
        FailureEvent { at_s: 3600.0, asset: "pump-p1".into() },
    ];
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("mode\tevent_times_h\tsurvived_days");
    let old = simulate_multi_service_longevity_with_load(
        &model,
        &["potable-water", "workshop-operability"],
        86_400.0,
        &events,
        battery,
        0.0,
    )
    .unwrap();
    let new = simulate_multi_service_longevity_event_driven(
        &model,
        &["potable-water", "workshop-operability"],
        86_400.0,
        &events,
        battery,
        0.0,
    )
    .unwrap();
    println!("instant-repair\t{:?}\t{:.0}", event_times(&old), old.survived_s / 86400.0);
    println!("event-driven\t{:?}\t{:.0}", event_times(&new), new.survived_s / 86400.0);
    ExitCode::SUCCESS
}
