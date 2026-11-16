// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase E sample: recycle-residue and adversarial-random.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{Flow, Model};
use poie_viability::engine::{run_engine, EngineOptions};
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn events() -> Vec<FailureEvent> {
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86_400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86_400.0, asset: "lathe-1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    events
}

fn canonical_days(model: &Model, failures: Vec<FailureEvent>, horizon_s: f64) -> f64 {
    let options = EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s,
        failures,
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        load_kw: 0.0,
    };
    run_engine(model, &options).unwrap().survived_s / 86_400.0
}

fn main() -> ExitCode {
    let base = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("experiment\trow\tlegacy\tcanonical\tclassification");
    for (bar_kg, residue_kg) in [(1.2_f64, 0.0_f64), (1.0, 0.2), (0.9, 0.3), (0.6, 0.6)] {
        let mut model = base.clone();
        let process = model.processes.iter_mut().find(|process| process.id == "recycle-bronze").unwrap();
        for output in &mut process.outputs {
            if output.resource == "bronze-bar" {
                output.si = bar_kg;
            }
        }
        if residue_kg > 0.0 {
            process.outputs.push(Flow { resource: "bronze-slag".into(), si: residue_kg });
        }
        let legacy = simulate_multi_service_longevity_with_limits(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &events(),
            battery,
        )
        .unwrap();
        let canonical = canonical_days(&model, events(), 90.0 * 86_400.0);
        let legacy_days = legacy.survived_s / 86_400.0;
        let class = if legacy_days.round() == canonical.round() { "EXPECTED_AGREEMENT" } else { "UNEXPLAINED" };
        println!("recycle-residue\tbar={bar_kg:.1},residue={residue_kg:.1}\t{legacy_days:.0}\t{canonical:.3}\t{class}");
    }

    let assets = ["pump-p1", "filter-1", "robot-m1", "pv-1", "lathe-1"];
    let trials = 500usize;
    let mut legacy_state = 0x9e37_79b9_7f4a_7c15u64;
    let mut canonical_state = 0x9e37_79b9_7f4a_7c15u64;
    let (mut legacy_zero, mut legacy_under30, mut legacy_under60) = (0usize, 0usize, 0usize);
    let (mut canonical_zero, mut canonical_under30, mut canonical_under60) = (0usize, 0usize, 0usize);
    let mut legacy_total = 0.0;
    let mut canonical_total = 0.0;
    let mut legacy_min = f64::INFINITY;
    let mut canonical_min = f64::INFINITY;
    for _ in 0..trials {
        let mask = 1 + (next(&mut legacy_state) % 31) as u32;
        let day = (next(&mut legacy_state) % 91) as f64;
        let failed: Vec<&str> =
            (0..assets.len()).filter(|index| mask & (1 << index) != 0).map(|index| assets[index]).collect();
        let legacy_events: Vec<FailureEvent> =
            failed.iter().map(|asset| FailureEvent { at_s: day * 86_400.0, asset: (*asset).into() }).collect();
        let legacy = simulate_multi_service_longevity_with_limits(
            &base,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &legacy_events,
            battery,
        )
        .unwrap()
        .survived_s
            / 86_400.0;
        let _ = next(&mut canonical_state);
        let _ = next(&mut canonical_state);
        let canonical = canonical_days(&base, legacy_events, 90.0 * 86_400.0);
        legacy_total += legacy;
        canonical_total += canonical;
        if legacy == 0.0 {
            legacy_zero += 1;
        }
        if legacy < 30.0 {
            legacy_under30 += 1;
        }
        if legacy < 60.0 {
            legacy_under60 += 1;
        }
        if canonical == 0.0 {
            canonical_zero += 1;
        }
        if canonical < 30.0 {
            canonical_under30 += 1;
        }
        if canonical < 60.0 {
            canonical_under60 += 1;
        }
        legacy_min = legacy_min.min(legacy);
        canonical_min = canonical_min.min(canonical);
    }
    let class = if legacy_zero == canonical_zero
        && legacy_under30 == canonical_under30
        && legacy_under60 == canonical_under60
    {
        "EXPECTED_AGREEMENT"
    } else {
        "SEMANTIC_CORRECTION_TIME_DISTRIBUTION"
    };
    println!("adversarial-rand\tstats\tzero={legacy_zero},under30={legacy_under30},under60={legacy_under60},mean={:.1},min={legacy_min:.1}\tzero={canonical_zero},under30={canonical_under30},under60={canonical_under60},mean={:.1},min={canonical_min:.1}\t{class}", legacy_total / trials as f64, canonical_total / trials as f64);
    ExitCode::SUCCESS
}
