// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2: regeneration horizon under long repairs and dense failures.

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

fn main() -> ExitCode {
    let root = root();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("model\tinterval_days\tinstant_days\tevent_driven_days");
    for model_name in ["base-05", "base-08"] {
        let mut model = Model::load(&root.join(format!("scenarios/{model_name}/model.toml")))
            .unwrap_or_else(|error| panic!("load {model_name}: {error}"));
        model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap().duration_s =
            Some(5.0 * 86400.0);
        for interval in [2_u32, 3, 5, 10] {
            let mut events: Vec<FailureEvent> = (1..=(89 / interval))
                .map(|index| FailureEvent { at_s: f64::from(index * interval) * 86400.0, asset: "pump-p1".into() })
                .collect();
            events.push(FailureEvent { at_s: 35.0 * 86400.0, asset: "lathe-1".into() });
            events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
            let old = simulate_multi_service_longevity_with_limits(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86400.0,
                &events,
                battery,
            )
            .unwrap();
            let new = simulate_multi_service_longevity_event_driven(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86400.0,
                &events,
                battery,
                0.0,
            )
            .unwrap();
            println!("{model_name}\t{interval}\t{:.0}\t{:.0}", old.survived_s / 86400.0, new.survived_s / 86400.0);
        }
    }
    ExitCode::SUCCESS
}
