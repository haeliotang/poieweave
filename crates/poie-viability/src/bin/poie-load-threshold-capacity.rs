// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 load regime switch vs battery capacity.

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
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86400.0, asset: "lathe-1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    println!("capacity_kwh\tfirst_energy_limited_load_kw");
    for capacity_kwh in [100.0, 200.0, 300.0] {
        let mut threshold = 0.0;
        for tenths in 20..=40 {
            let load_kw = f64::from(tenths) / 10.0;
            let report = simulate_multi_service_longevity_with_load(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86400.0,
                &events,
                Some(BatteryLimits { capacity_kwh, charge_kw: 10.0 }),
                load_kw,
            )
            .unwrap();
            if report.log.iter().any(|entry| entry.blocked_by.as_deref() == Some("energy_depleted")) {
                threshold = load_kw;
                break;
            }
        }
        println!("{capacity_kwh:.0}\t{threshold:.1}");
    }
    ExitCode::SUCCESS
}
