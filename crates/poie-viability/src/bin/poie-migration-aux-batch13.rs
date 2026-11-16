// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase E sample: adversarial-failures and adversarial-subsets.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions};
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
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

fn main() -> ExitCode {
    let model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let assets = ["pump-p1", "filter-1", "robot-m1", "pv-1", "lathe-1"];
    println!("experiment\trow\tlegacy_days\tcanonical_days\tclassification");
    let sets: Vec<Vec<&str>> = vec![
        vec![],
        vec!["pump-p1"],
        vec!["pump-p1", "filter-1"],
        vec!["pump-p1", "robot-m1"],
        vec!["pump-p1", "pv-1"],
        vec!["pump-p1", "lathe-1"],
        vec!["filter-1"],
        vec!["filter-1", "robot-m1"],
        vec!["filter-1", "pv-1"],
        vec!["filter-1", "lathe-1"],
        vec!["robot-m1"],
        vec!["robot-m1", "pv-1"],
        vec!["robot-m1", "lathe-1"],
        vec!["pv-1"],
        vec!["pv-1", "lathe-1"],
        vec!["lathe-1"],
    ];
    for failed in sets {
        let legacy = legacy_days(&model, &failed, 0.0);
        let canonical = canonical_days(&model, &failed, 0.0);
        let label = if failed.is_empty() { "baseline".to_string() } else { failed.join("+") };
        let class = if legacy.round() == canonical.round() {
            "EXPECTED_AGREEMENT"
        } else if label == "pump-p1+robot-m1" {
            "SEMANTIC_CORRECTION_REPAIR_EXECUTOR_AT_EVENT_TIME"
        } else {
            "UNEXPLAINED"
        };
        println!("adversarial-failures\t{label}\t{legacy:.0}\t{canonical:.3}\t{class}");
    }
    for at_day in [0.0_f64, 10.0] {
        let at_s = at_day * 86_400.0;
        let mut legacy_min = f64::INFINITY;
        let mut canonical_min = f64::INFINITY;
        let mut legacy_set = String::new();
        let mut canonical_set = String::new();
        for mask in 1u32..(1 << assets.len()) {
            let failed: Vec<&str> =
                (0..assets.len()).filter(|index| mask & (1 << index) != 0).map(|index| assets[index]).collect();
            let legacy = legacy_days(&model, &failed, at_s);
            let canonical = canonical_days(&model, &failed, at_s);
            if legacy < legacy_min || (legacy == legacy_min && failed.len() < legacy_set.split('+').count()) {
                legacy_min = legacy;
                legacy_set = failed.join("+");
            }
            if canonical < canonical_min
                || (canonical == canonical_min && failed.len() < canonical_set.split('+').count())
            {
                canonical_min = canonical;
                canonical_set = failed.join("+");
            }
        }
        let class = if legacy_min.round() == canonical_min.round() && legacy_set == canonical_set {
            "EXPECTED_AGREEMENT"
        } else if legacy_min.round() == canonical_min.round() {
            "SEMANTIC_CORRECTION_REPAIR_TIME"
        } else {
            "UNEXPLAINED"
        };
        println!("adversarial-subsets\tday={at_day:.0}\t{legacy_min:.0}:{legacy_set}\t{canonical_min:.3}:{canonical_set}\t{class}");
    }
    ExitCode::SUCCESS
}
