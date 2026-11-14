// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 random adversarial subsets under two time semantics.

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

fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
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
    let assets = ["pump-p1", "filter-1", "robot-m1", "pv-1", "lathe-1"];
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    let trials = 200usize;
    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    let mut old_zero = 0usize;
    let mut new_zero = 0usize;
    let mut old_total = 0.0;
    let mut new_total = 0.0;
    for _ in 0..trials {
        let mask = 1 + (next(&mut state) % 31) as u32;
        let mut events: Vec<FailureEvent> = (1..=15)
            .map(|index| FailureEvent { at_s: f64::from(index * 2) * 86_400.0, asset: "pump-p1".into() })
            .collect();
        events.extend(
            (0..assets.len())
                .filter(|index| mask & (1 << index) != 0)
                .map(|index| FailureEvent { at_s: 0.0, asset: assets[index].into() }),
        );
        let old = simulate_multi_service_longevity_with_limits(
            &model,
            &["potable-water", "workshop-operability"],
            30.0 * 86_400.0,
            &events,
            battery,
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
        let old_days = old.survived_s / 86_400.0;
        let new_days = new.survived_s / 86_400.0;
        old_total += old_days;
        new_total += new_days;
        if old_days == 0.0 {
            old_zero += 1;
        }
        if new_days == 0.0 {
            new_zero += 1;
        }
    }
    println!("trials\told_zero\tnew_zero\told_mean\tnew_mean");
    println!("{trials}\t{old_zero}\t{new_zero}\t{:.1}\t{:.1}", old_total / trials as f64, new_total / trials as f64);
    ExitCode::SUCCESS
}
