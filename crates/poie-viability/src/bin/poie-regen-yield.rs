// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 regeneration-yield sensitivity on Base-08.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn with_yield(mut model: Model, fraction: f64) -> Model {
    let process = model.processes.iter_mut().find(|process| process.id == "recycle-bronze").expect("recycle-bronze");
    for output in &mut process.outputs {
        if output.resource == "bronze-bar" {
            output.si *= fraction;
        }
    }
    model
}

fn main() -> ExitCode {
    let root = root();
    let base = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap_or_else(|error| {
        panic!("load Base-08: {error}");
    });
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("recycle_yield\tsurvived_days\tblocked_by");
    for yield_fraction in [1.0, 0.75, 0.5] {
        let model = with_yield(base.clone(), yield_fraction);
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
        println!("{yield_fraction:.2}\t{:.0}\t{blocked}", report.survived_s / 86400.0);
    }
    ExitCode::SUCCESS
}
