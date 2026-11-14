// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2: discrete external resupply deliveries extend a consumable-limited horizon.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{
    simulate_multi_service_longevity_with_load, simulate_multi_service_longevity_with_resupply, BatteryLimits,
    FailureEvent, Resupply,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let base = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86400.0, asset: "pump-p1".into() }).collect();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    let resupplies: Vec<Resupply> = [30.0, 60.0]
        .into_iter()
        .map(|day| Resupply { at_s: day * 86400.0, resource: "cutting-insert".into(), si: 1.0 })
        .collect();
    println!("mode\tinsert_stock\tsurvived_days\tblocked_by");
    for (mode, stock) in [("one-insert", 1.0), ("four-inserts", 4.0)] {
        let mut model = base.clone();
        model.lots.iter_mut().find(|lot| lot.id == "lot-insert").unwrap().si = stock;
        let report = simulate_multi_service_longevity_with_load(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86400.0,
            &events,
            battery,
            0.0,
        )
        .unwrap();
        let blocked = report.log.iter().find_map(|entry| entry.blocked_by.clone()).unwrap_or_else(|| "none".into());
        println!("{mode}-no-resupply\t{stock:.0}\t{:.0}\t{blocked}", report.survived_s / 86400.0);
    }
    let mut model = base.clone();
    model.lots.iter_mut().find(|lot| lot.id == "lot-insert").unwrap().si = 1.0;
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
    println!("one-insert-resupply\t1\t{:.0}\t{blocked}", report.survived_s / 86400.0);
    ExitCode::SUCCESS
}
