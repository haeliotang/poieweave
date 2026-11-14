// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 adversarial search over every simultaneous failure subset of five assets.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run(model: &Model, failed: &[&str]) -> (f64, String) {
    let events: Vec<FailureEvent> =
        failed.iter().map(|asset| FailureEvent { at_s: 0.0, asset: (*asset).into() }).collect();
    let report = simulate_multi_service_longevity_with_limits(
        model,
        &["potable-water", "workshop-operability"],
        60.0 * 86400.0,
        &events,
        Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
    )
    .unwrap();
    let blocked = report.log.iter().find_map(|entry| entry.blocked_by.clone()).unwrap_or_else(|| "none".into());
    (report.survived_s / 86400.0, blocked)
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let assets = ["pump-p1", "filter-1", "robot-m1", "pv-1", "lathe-1"];
    println!("subsets\tmin_days\tminimal_size\tminimal_set");
    let mut min_days = f64::INFINITY;
    let mut minimal: (usize, String) = (assets.len() + 1, String::new());
    for mask in 1u32..(1 << assets.len()) {
        let failed: Vec<&str> =
            (0..assets.len()).filter(|index| mask & (1 << index) != 0).map(|index| assets[index]).collect();
        let (days, _blocked) = run(&model, &failed);
        if days < min_days || (days == min_days && failed.len() < minimal.0) {
            min_days = days;
            minimal = (failed.len(), failed.join("+"));
        }
    }
    println!("{}\t{min_days:.0}\t{}\t{}", (1 << assets.len()) - 1, minimal.0, minimal.1);
    ExitCode::SUCCESS
}
