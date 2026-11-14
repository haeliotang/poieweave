// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2: compare the load regime switch with average PV recharge power.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_multi_service_longevity_with_load, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let recharge = model.processes.iter().find(|process| process.id == "recharge-energy").expect("recharge-energy");
    let hours = recharge.duration_s.expect("duration") / 3600.0;
    let kwh = recharge.outputs.iter().find(|output| output.resource == "electric-energy").expect("energy output").si
        / 3_600_000.0;
    let average_kw = kwh / hours;
    let mut threshold = 0.0;
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86400.0, asset: "lathe-1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    for tenths in 20..=40 {
        let load_kw = f64::from(tenths) / 10.0;
        let report = simulate_multi_service_longevity_with_load(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86400.0,
            &events,
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
            load_kw,
        )
        .unwrap();
        if report.log.iter().any(|entry| entry.blocked_by.as_deref() == Some("energy_depleted")) {
            threshold = load_kw;
            break;
        }
    }
    println!("recharge_output_kwh={kwh:.1}");
    println!("recharge_duration_h={hours:.1}");
    println!("average_recharge_kw={average_kw:.2}");
    println!("first_energy_limited_load_kw={threshold:.1}");
    println!("delta_kw={:.2}", threshold - average_kw);
    ExitCode::SUCCESS
}
