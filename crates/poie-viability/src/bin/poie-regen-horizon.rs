// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 regeneration effect: Base-05 vs Base-08 monthly pump failures.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("model\tinterval_days\tsurvived_days\tblocked_by");
    for model_name in ["base-05", "base-08"] {
        let model = Model::load(&root.join(format!("scenarios/{model_name}/model.toml")))
            .unwrap_or_else(|error| panic!("load {model_name}: {error}"));
        for interval in [10_u32, 15, 20, 30] {
            let mut events: Vec<FailureEvent> = (1..=(89 / interval))
                .map(|index| FailureEvent { at_s: f64::from(index * interval) * 86400.0, asset: "pump-p1".into() })
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
            println!("{model_name}\t{interval}\t{:.0}\t{blocked}", report.survived_s / 86400.0);
        }
    }
    ExitCode::SUCCESS
}
