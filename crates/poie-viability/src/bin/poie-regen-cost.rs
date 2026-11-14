// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 regeneration cost sensitivity on Base-08.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn with_cost(mut model: Model, duration_multiplier: f64, energy_multiplier: f64) -> Model {
    let process = model.processes.iter_mut().find(|process| process.id == "recycle-bronze").expect("recycle-bronze");
    if let Some(duration) = process.duration_s.as_mut() {
        *duration *= duration_multiplier;
    }
    for input in &mut process.inputs {
        if input.resource == "electric-energy" {
            input.si *= energy_multiplier;
        }
    }
    model
}

fn main() -> ExitCode {
    let root = root();
    let base = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("duration_x\tenergy_x\tsurvived_days\tblocked_by");
    for duration_multiplier in [1.0, 2.0, 4.0] {
        for energy_multiplier in [1.0, 2.0, 100.0] {
            let model = with_cost(base.clone(), duration_multiplier, energy_multiplier);
            let mut events: Vec<FailureEvent> = (1..=8)
                .map(|index| FailureEvent { at_s: f64::from(index * 10) * 86400.0, asset: "pump-p1".into() })
                .collect();
            events.push(FailureEvent { at_s: 35.0 * 86400.0, asset: "lathe-1".into() });
            events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
            let report = simulate_multi_service_longevity_with_limits(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86400.0,
                &events,
                battery,
            )
            .unwrap();
            let blocked = report.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
            println!("{duration_multiplier:.1}\t{energy_multiplier:.1}\t{:.0}\t{blocked}", report.survived_s / 86400.0);
        }
    }
    ExitCode::SUCCESS
}
