// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 adversarial subsets with and without external resupply.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{
    simulate_multi_service_longevity_with_limits, simulate_multi_service_longevity_with_resupply, BatteryLimits,
    FailureEvent, Resupply,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run(model: &Model, failed: &[&str]) -> (f64, f64) {
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86400.0, asset: "pump-p1".into() }).collect();
    events.extend(failed.iter().map(|asset| FailureEvent { at_s: 0.0, asset: (*asset).into() }));
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    let no_supply = simulate_multi_service_longevity_with_limits(
        model,
        &["potable-water", "workshop-operability"],
        90.0 * 86400.0,
        &events,
        battery,
    )
    .unwrap();
    let resupplies: Vec<Resupply> = (1..=8)
        .map(|index| Resupply { at_s: f64::from(index * 10) * 86400.0, resource: "cutting-insert".into(), si: 1.0 })
        .collect();
    let with_supply = simulate_multi_service_longevity_with_resupply(
        model,
        &["potable-water", "workshop-operability"],
        90.0 * 86400.0,
        &events,
        &resupplies,
        battery,
        0.0,
    )
    .unwrap();
    (no_supply.survived_s / 86400.0, with_supply.survived_s / 86400.0)
}

fn main() -> ExitCode {
    let root = root();
    let mut model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    model.lots.iter_mut().find(|lot| lot.id == "lot-insert").unwrap().si = 1.0;
    let assets = ["pump-p1", "filter-1", "robot-m1", "pv-1", "lathe-1"];
    println!("at_day\tsubsets\tno_supply_min\twith_supply_min\tminimal_size\tminimal_set");
    for at_day in [0.0, 10.0] {
        let mut min_no = f64::INFINITY;
        let mut min_yes = f64::INFINITY;
        let mut minimal: (usize, String) = (assets.len() + 1, String::new());
        for mask in 1u32..(1 << assets.len()) {
            let failed: Vec<&str> =
                (0..assets.len()).filter(|index| mask & (1 << index) != 0).map(|index| assets[index]).collect();
            let (no, yes) = run(&model, &failed);
            min_no = min_no.min(no);
            if yes < min_yes {
                min_yes = yes;
                minimal = (failed.len(), failed.join("+"));
            }
        }
        println!("{at_day:.0}\t{}\t{min_no:.0}\t{min_yes:.0}\t{}\t{}", (1 << assets.len()) - 1, minimal.0, minimal.1);
    }
    ExitCode::SUCCESS
}
