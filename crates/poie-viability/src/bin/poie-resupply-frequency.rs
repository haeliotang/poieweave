// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2: resupply interval sweep for a consumable-limited base.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_multi_service_longevity_with_resupply, BatteryLimits, FailureEvent, Resupply};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let base = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86400.0, asset: "pump-p1".into() }).collect();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("interval_days\tsurvived_days\tblocked_by");
    for interval in [10.0_f64, 20.0, 30.0, 60.0] {
        let mut model = base.clone();
        model.lots.iter_mut().find(|lot| lot.id == "lot-insert").unwrap().si = 1.0;
        let mut resupplies = Vec::new();
        let mut day = interval;
        while day < 90.0 {
            resupplies.push(Resupply { at_s: day * 86400.0, resource: "cutting-insert".into(), si: 1.0 });
            day += interval;
        }
        let report = simulate_multi_service_longevity_with_resupply(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86400.0,
            &events,
            &resupplies,
            battery,
            0.0,
        )
        .unwrap();
        let blocked = report.log.iter().find_map(|entry| entry.blocked_by.clone()).unwrap_or_else(|| "none".into());
        println!("{interval:.0}\t{:.0}\t{blocked}", report.survived_s / 86400.0);
    }
    ExitCode::SUCCESS
}
