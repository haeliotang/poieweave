// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase E sample: failure envelope/pairs coverage and regen threshold.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions, EngineTermination};
use poie_viability::survival::{
    failure_envelope, simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn coverage(model: &Model, max_failures: usize) -> (usize, usize, usize, usize) {
    let envelope = failure_envelope(model, "potable-water", 12.0 * 3600.0, max_failures).unwrap();
    let mut legacy = 0;
    let mut canonical = 0;
    for case in &envelope.cases {
        if case.recoverable {
            legacy += 1;
        }
        let events: Vec<FailureEvent> =
            case.failed.iter().map(|asset| FailureEvent { at_s: 0.0, asset: asset.clone() }).collect();
        let options = EngineOptions {
            services: vec!["potable-water"],
            horizon_s: 12.0 * 3600.0,
            failures: events,
            resupplies: Vec::new(),
            resources: vec!["robot-m1".into(), "lathe-1".into()],
            alternative_pool: vec!["robot-m1".into()],
            battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
            load_kw: 0.0,
        };
        if run_engine(model, &options).unwrap().termination == EngineTermination::CompletedHorizon {
            canonical += 1;
        }
    }
    (legacy, canonical, envelope.cases.len(), envelope.recoverable)
}

fn threshold(base: &Model) -> (u32, u32, u32, u32) {
    let mut legacy_safe = 1;
    let mut legacy_blocked = 100;
    let mut canonical_safe = 1;
    let mut canonical_blocked = 100;
    for multiplier in 1u32..=100 {
        let mut model = base.clone();
        model.lots.retain(|lot| !lot.id.starts_with("lot-sleeve") && lot.id != "lot-bronze");
        let process = model.processes.iter_mut().find(|process| process.id == "recycle-bronze").unwrap();
        for input in &mut process.inputs {
            if input.resource == "electric-energy" {
                input.si *= f64::from(multiplier);
            }
        }
        let mut events: Vec<FailureEvent> = (1..=8)
            .map(|index| FailureEvent { at_s: f64::from(index * 10) * 86_400.0, asset: "pump-p1".into() })
            .collect();
        events.push(FailureEvent { at_s: 35.0 * 86_400.0, asset: "lathe-1".into() });
        events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
        let legacy = simulate_multi_service_longevity_with_limits(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &events,
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        )
        .unwrap();
        let legacy_days = legacy.survived_s / 86_400.0;
        if legacy_days >= 10.0 {
            legacy_safe = multiplier;
        } else if multiplier < legacy_blocked {
            legacy_blocked = multiplier;
        }
        let options = EngineOptions {
            services: vec!["potable-water", "workshop-operability"],
            horizon_s: 90.0 * 86_400.0,
            failures: events,
            resupplies: Vec::new(),
            resources: vec!["robot-m1".into(), "lathe-1".into()],
            alternative_pool: vec!["robot-m1".into()],
            battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
            load_kw: 0.0,
        };
        let canonical_days = run_engine(&model, &options).unwrap().survived_s / 86_400.0;
        if canonical_days >= 10.0 {
            canonical_safe = multiplier;
        } else if multiplier < canonical_blocked {
            canonical_blocked = multiplier;
        }
    }
    (legacy_safe, legacy_blocked, canonical_safe, canonical_blocked)
}

fn main() -> ExitCode {
    let base05 = Model::load(&root().join("scenarios/base-05/model.toml")).unwrap();
    let base08 = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    println!("experiment\tmetric\tlegacy\tcanonical\ttotal\tclassification");
    for (label, max_failures) in [("failure-envelope", 1usize), ("failure-pairs", 2)] {
        let (legacy, canonical, total, _) = coverage(&base05, max_failures);
        let class = if legacy == canonical { "EXPECTED_AGREEMENT" } else { "SEMANTIC_CORRECTION_COVERAGE_DEFINITION" };
        println!("{label}\trecoverable\t{legacy}\t{canonical}\t{total}\t{class}");
    }
    let (legacy_safe, legacy_blocked, canonical_safe, canonical_blocked) = threshold(&base08);
    println!(
        "regen-threshold\tmax_safe\t{legacy_safe}\t{canonical_safe}\t-\t{}",
        if legacy_safe == canonical_safe { "EXPECTED_AGREEMENT" } else { "UNEXPLAINED" }
    );
    println!(
        "regen-threshold\tfirst_blocked\t{legacy_blocked}\t{canonical_blocked}\t-\t{}",
        if legacy_blocked == canonical_blocked { "EXPECTED_AGREEMENT" } else { "UNEXPLAINED" }
    );
    ExitCode::SUCCESS
}
