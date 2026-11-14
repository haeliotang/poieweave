// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 does increasing recharge output move the 2.6 kW horizon?

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_multi_service_longevity_with_load, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let base = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86400.0, asset: "lathe-1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    println!("recharge_kwh\taverage_kw\tsurvived_days\tblocked_by");
    for kwh in [20.0, 30.0, 40.0] {
        let mut model = base.clone();
        let process = model.processes.iter_mut().find(|process| process.id == "recharge-energy").unwrap();
        for output in &mut process.outputs {
            if output.resource == "electric-energy" {
                output.si = kwh * 3.6e6;
            }
        }
        let report = simulate_multi_service_longevity_with_load(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86400.0,
            &events,
            battery,
            2.6,
        )
        .unwrap();
        let blocked = report.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
        println!("{kwh:.0}\t{:.2}\t{:.0}\t{blocked}", kwh / 8.0, report.survived_s / 86400.0);
    }
    ExitCode::SUCCESS
}
