// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 capacity/charge interventions exactly at the energy switch.

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
    println!("load_kw\tcapacity_kwh\tcharge_kw\tsurvived_days\tblocked_by");
    for load_kw in [2.6] {
        for capacity_kwh in [100.0, 200.0, 300.0] {
            for charge_kw in [3.0, 10.0, 30.0] {
                let report = simulate_multi_service_longevity_with_load(
                    &model,
                    &["potable-water", "workshop-operability"],
                    90.0 * 86400.0,
                    &events,
                    Some(BatteryLimits { capacity_kwh, charge_kw }),
                    load_kw,
                )
                .unwrap();
                let blocked = report.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
                println!(
                    "{load_kw:.1}\t{capacity_kwh:.0}\t{charge_kw:.0}\t{:.0}\t{blocked}",
                    report.survived_s / 86400.0
                );
            }
        }
    }
    ExitCode::SUCCESS
}
