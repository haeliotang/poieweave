// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase D: canonicalise the four not-migrated certificate metric groups.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions, EngineTermination};
use poie_viability::survival::{
    failure_envelope, simulate_multi_service_longevity_with_limits, simulate_multi_service_longevity_with_load,
    BatteryLimits, FailureEvent,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn canonical_days(model: &Model, horizon_s: f64, events: Vec<FailureEvent>, load_kw: f64) -> (f64, EngineTermination) {
    let options = EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s,
        failures: events,
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        load_kw,
    };
    let outcome = run_engine(model, &options).unwrap();
    (outcome.survived_s / 86_400.0, outcome.termination)
}

fn horizon_events() -> Vec<FailureEvent> {
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86_400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86_400.0, asset: "lathe-1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    events
}

fn main() -> ExitCode {
    let base05 = Model::load(&root().join("scenarios/base-05/model.toml")).unwrap();
    let base08 = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });

    // 1. recovery coverage, Base-05 single-failure envelope.
    let envelope = failure_envelope(&base05, "potable-water", 12.0 * 3600.0, 1).unwrap();
    let mut legacy_cov = 0;
    let mut canonical_cov = 0;
    for case in &envelope.cases {
        if case.failed.is_empty() {
            if case.recoverable {
                legacy_cov += 1;
                canonical_cov += 1;
            }
            continue;
        }
        if case.recoverable {
            legacy_cov += 1;
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
            battery,
            load_kw: 0.0,
        };
        let outcome = run_engine(&base05, &options).unwrap();
        let canonical_ok = outcome.termination == EngineTermination::CompletedHorizon;
        if canonical_ok {
            canonical_cov += 1;
        }
        if case.recoverable != canonical_ok {
            println!("coverage_case\t{}\tlegacy={}\tcanonical={canonical_ok}", case.failed.join("+"), case.recoverable);
        }
    }
    println!("coverage\tlegacy={legacy_cov}\tcanonical={canonical_cov}\ttotal={}", envelope.cases.len());

    // 2. regenerative capacity: canonical Base-05 vs Base-08 horizon.
    let h05 = canonical_days(&base05, 90.0 * 86_400.0, horizon_events(), 0.0).0;
    let h08 = canonical_days(&base08, 90.0 * 86_400.0, horizon_events(), 0.0).0;
    println!("regenerative_capacity_days\tlegacy={:.0}\tcanonical={:.1}", 20.0, h08 - h05);

    // 3. yield sensitivity: legacy vs canonical.
    println!("yield\tlegacy_days\tcanonical_days");
    for fraction in [1.0_f64, 0.8, 0.5] {
        let mut model = base08.clone();
        let process = model.processes.iter_mut().find(|process| process.id == "recycle-bronze").unwrap();
        for output in &mut process.outputs {
            if output.resource == "bronze-bar" {
                output.si *= fraction;
            }
        }
        let legacy = simulate_multi_service_longevity_with_limits(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &horizon_events(),
            battery,
        )
        .unwrap();
        let canonical = canonical_days(&model, 90.0 * 86_400.0, horizon_events(), 0.0).0;
        println!("{fraction:.1}\t{:.0}\t{canonical:.0}", legacy.survived_s / 86_400.0);
    }

    // 4. recharge boundary: first energy-limited load, legacy vs canonical.
    let recharge = base08.processes.iter().find(|process| process.id == "recharge-energy").unwrap();
    let hours = recharge.duration_s.unwrap() / 3600.0;
    let kwh = recharge.outputs.iter().find(|output| output.resource == "electric-energy").unwrap().si / 3_600_000.0;
    let average_kw = kwh / hours;
    let mut legacy_threshold = 0.0;
    let mut canonical_threshold = 0.0;
    for tenths in 20..=40 {
        let load_kw = f64::from(tenths) / 10.0;
        let legacy = simulate_multi_service_longevity_with_load(
            &base08,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &horizon_events(),
            battery,
            load_kw,
        )
        .unwrap();
        if legacy_threshold == 0.0
            && legacy.log.iter().any(|entry| entry.blocked_by.as_deref() == Some("energy_depleted"))
        {
            legacy_threshold = load_kw;
        }
        let (_, termination) = canonical_days(&base08, 90.0 * 86_400.0, horizon_events(), load_kw);
        if canonical_threshold == 0.0
            && matches!(&termination, EngineTermination::Blocked(reason) if reason.contains("energy_depleted"))
        {
            canonical_threshold = load_kw;
        }
    }
    println!("recharge\taverage_kw={average_kw:.2}\tlegacy_threshold={legacy_threshold:.1}\tcanonical_threshold={canonical_threshold:.1}");
    ExitCode::SUCCESS
}
