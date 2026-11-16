// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase E sample: load and load thresholds by capacity/charge.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions};
use poie_viability::survival::{simulate_multi_service_longevity_with_load, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn events() -> Vec<FailureEvent> {
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86_400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86_400.0, asset: "lathe-1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    events
}

fn canonical(model: &Model, load_kw: f64, capacity: f64, charge_kw: f64) -> (f64, bool) {
    let options = EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s: 90.0 * 86_400.0,
        failures: events(),
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(BatteryLimits { capacity_kwh: capacity, charge_kw }),
        load_kw,
    };
    let outcome = run_engine(model, &options).unwrap();
    let limited = format!("{:?}", outcome.termination).contains("energy_depleted");
    (outcome.survived_s / 86_400.0, limited)
}

fn main() -> ExitCode {
    let model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    println!("experiment\trow\tlegacy_days\tcanonical_days\tclassification");
    let base05 = Model::load(&root().join("scenarios/base-05/model.toml")).unwrap();
    for load_kw in [0.0_f64, 1.0, 2.0, 3.0, 4.0] {
        let legacy = simulate_multi_service_longevity_with_load(
            &base05,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &events(),
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
            load_kw,
        )
        .unwrap();
        let (canonical_days, _) = canonical(&base05, load_kw, 200.0, 10.0);
        let legacy_days = legacy.survived_s / 86_400.0;
        let class = if legacy_days.round() == canonical_days.round() { "EXPECTED_AGREEMENT" } else { "UNEXPLAINED" };
        println!("load\tload={load_kw:.1}\t{legacy_days:.0}\t{canonical_days:.3}\t{class}");
    }
    for capacity in [100.0_f64, 200.0, 300.0] {
        let mut legacy_threshold = 0.0;
        let mut canonical_threshold = 0.0;
        for tenths in 20..=40 {
            let load_kw = f64::from(tenths) / 10.0;
            let legacy = simulate_multi_service_longevity_with_load(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86_400.0,
                &events(),
                Some(BatteryLimits { capacity_kwh: capacity, charge_kw: 10.0 }),
                load_kw,
            )
            .unwrap();
            if legacy_threshold == 0.0
                && legacy.log.iter().any(|entry| entry.blocked_by.as_deref() == Some("energy_depleted"))
            {
                legacy_threshold = load_kw;
            }
            let (_, limited) = canonical(&model, load_kw, capacity, 10.0);
            if canonical_threshold == 0.0 && limited {
                canonical_threshold = load_kw;
            }
        }
        let class =
            if (legacy_threshold - canonical_threshold).abs() < 1e-9 { "EXPECTED_AGREEMENT" } else { "UNEXPLAINED" };
        println!(
            "load-threshold-capacity\tcapacity={capacity:.0}\t{legacy_threshold:.1}\t{canonical_threshold:.1}\t{class}"
        );
    }
    for charge_kw in [3.0_f64, 10.0, 30.0] {
        let mut legacy_threshold = 0.0;
        let mut canonical_threshold = 0.0;
        for tenths in 20..=40 {
            let load_kw = f64::from(tenths) / 10.0;
            let legacy = simulate_multi_service_longevity_with_load(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86_400.0,
                &events(),
                Some(BatteryLimits { capacity_kwh: 200.0, charge_kw }),
                load_kw,
            )
            .unwrap();
            if legacy_threshold == 0.0
                && legacy.log.iter().any(|entry| entry.blocked_by.as_deref() == Some("energy_depleted"))
            {
                legacy_threshold = load_kw;
            }
            let (_, limited) = canonical(&model, load_kw, 200.0, charge_kw);
            if canonical_threshold == 0.0 && limited {
                canonical_threshold = load_kw;
            }
        }
        let class =
            if (legacy_threshold - canonical_threshold).abs() < 1e-9 { "EXPECTED_AGREEMENT" } else { "UNEXPLAINED" };
        println!(
            "load-threshold-charge\tcharge={charge_kw:.0}\t{legacy_threshold:.1}\t{canonical_threshold:.1}\t{class}"
        );
    }
    ExitCode::SUCCESS
}
