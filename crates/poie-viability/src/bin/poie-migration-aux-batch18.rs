// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase E sample: adversarial-timing.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions};
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn legacy_days(model: &Model, failed: &[&str], at_s: f64) -> f64 {
    let events: Vec<FailureEvent> = failed.iter().map(|asset| FailureEvent { at_s, asset: (*asset).into() }).collect();
    simulate_multi_service_longevity_with_limits(
        model,
        &["potable-water", "workshop-operability"],
        60.0 * 86_400.0,
        &events,
        Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
    )
    .unwrap()
    .survived_s
        / 86_400.0
}

fn canonical_days(model: &Model, failed: &[&str], at_s: f64) -> f64 {
    let events: Vec<FailureEvent> = failed.iter().map(|asset| FailureEvent { at_s, asset: (*asset).into() }).collect();
    let options = EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s: 60.0 * 86_400.0,
        failures: events,
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        load_kw: 0.0,
    };
    run_engine(model, &options).unwrap().survived_s / 86_400.0
}

fn main() -> ExitCode {
    let model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let assets = ["pump-p1", "filter-1", "robot-m1", "pv-1", "lathe-1"];
    println!("experiment\tat_day\tset\tsize\tlegacy_days\tcanonical_days\tclassification");
    let mut semantic = 0usize;
    let mut rows = 0usize;
    for at_day in [0.0_f64, 10.0] {
        let at_s = at_day * 86_400.0;
        let mut sets: Vec<Vec<&str>> = vec![vec![]];
        for (index, asset) in assets.iter().enumerate() {
            sets.push(vec![*asset]);
            for other in &assets[index + 1..] {
                sets.push(vec![*asset, *other]);
            }
        }
        for failed in sets {
            let label = if failed.is_empty() { "baseline".to_string() } else { failed.join("+") };
            let legacy = legacy_days(&model, &failed, at_s);
            let canonical = canonical_days(&model, &failed, at_s);
            let class = if legacy.round() == canonical.round() {
                "EXPECTED_AGREEMENT"
            } else if failed.contains(&"robot-m1") || failed.contains(&"filter-1") {
                "SEMANTIC_CORRECTION_REPAIR_EXECUTOR_AT_EVENT_TIME"
            } else {
                "UNEXPLAINED"
            };
            if class != "EXPECTED_AGREEMENT" {
                semantic += 1;
            }
            rows += 1;
            println!(
                "adversarial-timing\t{at_day:.0}\t{label}\t{}\t{legacy:.0}\t{canonical:.3}\t{class}",
                failed.len()
            );
        }
    }
    eprintln!("summary\trows={rows}\tsemantic={semantic}");
    ExitCode::SUCCESS
}
