// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 forced recycle energy threshold vs initial electric-energy stock.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn horizon_for(base: &Model, multiplier: u32, initial_joules: f64) -> f64 {
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
        Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
    )
    .unwrap()
    .survived_s
        / 86400.0
}

fn first_blocked(base: &Model, initial_joules: f64) -> u32 {
    for multiplier in 1u32..=300 {
        if horizon_for(base, multiplier, initial_joules) < 10.0 {
            return multiplier;
        }
    }
    0
}

fn main() -> ExitCode {
    let root = root();
    let base = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    println!("initial_kwh\tfirst_blocked_x");
    for initial_kwh in [20.0, 40.0, 60.0, 80.0, 120.0] {
        let blocked = first_blocked(&base, initial_kwh * 3.6e6);
        println!("{initial_kwh:.0}\t{blocked}");
    }
    ExitCode::SUCCESS
}
