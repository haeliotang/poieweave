// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 does battery capacity cap the initial energy stock?

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn horizon_for(base: &Model, multiplier: u32, capacity_kwh: f64, initial_joules: f64) -> f64 {
    let mut model = base.clone();
    model.lots.retain(|lot| !lot.id.starts_with("lot-sleeve") && lot.id != "lot-bronze");
    if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "electric-energy") {
        lot.si = initial_joules;
    }
    let process = model.processes.iter_mut().find(|process| process.id == "recycle-bronze").unwrap();
    for input in &mut process.inputs {
        if input.resource == "electric-energy" {
            input.si *= f64::from(multiplier);
        }
    }
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86400.0, asset: "lathe-1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    simulate_multi_service_longevity_with_limits(
        &model,
        &["potable-water", "workshop-operability"],
        90.0 * 86400.0,
        &events,
        Some(BatteryLimits { capacity_kwh, charge_kw: 10.0 }),
    )
    .unwrap()
    .survived_s
        / 86400.0
}

fn first_blocked(base: &Model, capacity_kwh: f64, initial_joules: f64) -> u32 {
    for multiplier in 1u32..=700 {
        if horizon_for(base, multiplier, capacity_kwh, initial_joules) < 10.0 {
            return multiplier;
        }
    }
    0
}

fn main() -> ExitCode {
    let root = root();
    let base = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    println!("initial_kwh\tcapacity_kwh\tfirst_blocked_x");
    for capacity_kwh in [200.0, 400.0, 600.0] {
        let blocked = first_blocked(&base, capacity_kwh, 500.0 * 3.6e6);
        println!("500\t{capacity_kwh:.0}\t{blocked}");
    }
    ExitCode::SUCCESS
}
