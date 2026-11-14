// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 adversarial subsets under long repairs and two time semantics.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{
    simulate_multi_service_longevity_event_driven, simulate_multi_service_longevity_with_limits, BatteryLimits,
    FailureEvent,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run(model: &Model, failed: &[&str]) -> (f64, f64) {
    let mut events: Vec<FailureEvent> =
        (1..=45).map(|index| FailureEvent { at_s: f64::from(index * 2) * 86_400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86_400.0, asset: "lathe-1".into() });
    events.extend(failed.iter().map(|asset| FailureEvent { at_s: 0.0, asset: (*asset).into() }));
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    let old = simulate_multi_service_longevity_with_limits(
        model,
        &["potable-water", "workshop-operability"],
        90.0 * 86400.0,
        &events,
        battery,
    )
    .unwrap();
    let new = simulate_multi_service_longevity_event_driven(
        model,
        &["potable-water", "workshop-operability"],
        90.0 * 86400.0,
        &events,
        battery,
        0.0,
    )
    .unwrap();
    (old.survived_s / 86400.0, new.survived_s / 86400.0)
}

fn main() -> ExitCode {
    let root = root();
    let mut model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap().duration_s =
        Some(5.0 * 86_400.0);
    let assets = ["pump-p1", "filter-1", "robot-m1", "pv-1", "lathe-1"];
    println!("at_day\tsubsets\tinstant_min\tevent_min\tminimal_size\tminimal_set");
    for at_day in [0.0, 10.0] {
        let mut min_old = f64::INFINITY;
        let mut min_new = f64::INFINITY;
        let mut minimal: (usize, String) = (assets.len() + 1, String::new());
        for mask in 1u32..(1 << assets.len()) {
            let failed: Vec<&str> =
                (0..assets.len()).filter(|index| mask & (1 << index) != 0).map(|index| assets[index]).collect();
            let (old, new) = run(&model, &failed);
            if old < min_old {
                min_old = old;
            }
            if new < min_new {
                min_new = new;
            }
            if new < min_new || (new == min_new && failed.len() < minimal.0) {
                minimal = (failed.len(), failed.join("+"));
            }
        }
        println!("{at_day:.0}\t{}\t{min_old:.0}\t{min_new:.0}\t{}\t{}", (1 << assets.len()) - 1, minimal.0, minimal.1);
    }
    ExitCode::SUCCESS
}
