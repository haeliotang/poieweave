// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2: can clock-advanced repair time change the survival horizon?

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

fn main() -> ExitCode {
    let root = root();
    let mut model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    for lot in &mut model.lots {
        if lot.resource == "sleeve-p1" {
            lot.si = 0.0;
        }
        if lot.resource == "grease" {
            lot.si = 0.15;
        }
    }
    model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap().duration_s =
        Some(5.0 * 86_400.0);
    let events: Vec<FailureEvent> =
        (1..=15).map(|index| FailureEvent { at_s: f64::from(index * 2) * 86_400.0, asset: "pump-p1".into() }).collect();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("mode\tsurvived_days\tblocked_by");
    let old = simulate_multi_service_longevity_with_load(
        &model,
        &["potable-water", "workshop-operability"],
        30.0 * 86_400.0,
        &events,
        battery,
        0.0,
    )
    .unwrap();
    let new = simulate_multi_service_longevity_event_driven(
        &model,
        &["potable-water", "workshop-operability"],
        30.0 * 86_400.0,
        &events,
        battery,
        0.0,
    )
    .unwrap();
    let old_blocked = old.log.iter().find_map(|entry| entry.blocked_by.clone()).unwrap_or_else(|| "none".into());
    let new_blocked = new.log.iter().find_map(|entry| entry.blocked_by.clone()).unwrap_or_else(|| "none".into());
    println!("instant-repair\t{:.0}\t{old_blocked}", old.survived_s / 86400.0);
    println!("event-driven\t{:.0}\t{new_blocked}", new.survived_s / 86400.0);
    ExitCode::SUCCESS
}
