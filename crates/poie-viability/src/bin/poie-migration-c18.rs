// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase B: C18 random adversarial subsets, legacy vs canonical.
//! Seed: 0x9e3779b97f4a7c15, xorshift64 (13,7,17). 200 trials.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions};
use poie_viability::survival::{
    simulate_multi_service_longevity_event_driven, simulate_multi_service_longevity_with_limits, BatteryLimits,
    FailureEvent,
};

const SEED: u64 = 0x9e37_79b9_7f4a_7c15;
const TRIALS: usize = 200;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn quantiles(values: &mut [f64], probs: &[f64]) -> Vec<f64> {
    values.sort_by(f64::total_cmp);
    probs
        .iter()
        .map(|p| {
            let index = ((values.len() - 1) as f64 * p).round() as usize;
            values[index]
        })
        .collect()
}

fn main() -> ExitCode {
    let mut model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    for lot in &mut model.lots {
        if lot.resource == "sleeve-p1" {
            lot.si = 0.0;
        }
        if lot.resource == "grease" {
            lot.si = 0.15;
        }
    }
    model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap().duration_s =
        Some(5.0 * 86_400.0);
    let assets = ["pump-p1", "filter-1", "robot-m1", "pv-1", "lathe-1"];
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    let mut state = SEED;
    let mut old_values = Vec::new();
    let mut new_values = Vec::new();
    let mut canonical_values = Vec::new();
    let mut old_zero = 0usize;
    let mut new_zero = 0usize;
    let mut canonical_zero = 0usize;
    for _ in 0..TRIALS {
        let mask = 1 + (next(&mut state) % 31) as u32;
        let mut events: Vec<FailureEvent> = (1..=15)
            .map(|index| FailureEvent { at_s: f64::from(index * 2) * 86_400.0, asset: "pump-p1".into() })
            .collect();
        events.extend(
            (0..assets.len())
                .filter(|index| mask & (1 << index) != 0)
                .map(|index| FailureEvent { at_s: 0.0, asset: assets[index].into() }),
        );
        let old = simulate_multi_service_longevity_with_limits(
            &model,
            &["potable-water", "workshop-operability"],
            30.0 * 86_400.0,
            &events,
            battery,
        )
        .unwrap();
        let new = simulate_multi_service_longevity_event_driven(
            &model,
            &["potable-water", "workshop-operability"],
            30.0 * 86_400.0,
            &events,
            battery,
            0.0,
        )
        .unwrap();
        let options = EngineOptions {
            services: vec!["potable-water", "workshop-operability"],
            horizon_s: 30.0 * 86_400.0,
            failures: events,
            resupplies: Vec::new(),
            resources: vec!["robot-m1".into(), "lathe-1".into()],
            alternative_pool: vec!["robot-m1".into()],
            battery,
            load_kw: 0.0,
        };
        let canonical = run_engine(&model, &options).unwrap();
        let old_days = old.survived_s / 86_400.0;
        let new_days = new.survived_s / 86_400.0;
        let canonical_days = canonical.survived_s / 86_400.0;
        if old_days == 0.0 {
            old_zero += 1;
        }
        if new_days == 0.0 {
            new_zero += 1;
        }
        if canonical_days == 0.0 {
            canonical_zero += 1;
        }
        old_values.push(old_days);
        new_values.push(new_days);
        canonical_values.push(canonical_days);
    }
    let mean = |values: &[f64]| values.iter().sum::<f64>() / values.len() as f64;
    println!("seed_chosen\t{TRIALS}\t{TRIALS}\t{TRIALS}");
    println!("trials\told_zero\tnew_zero\tcanonical_zero\told_mean\tnew_mean\tcanonical_mean");
    println!(
        "{TRIALS}\t{old_zero}\t{new_zero}\t{canonical_zero}\t{:.1}\t{:.1}\t{:.1}",
        mean(&old_values),
        mean(&new_values),
        mean(&canonical_values)
    );
    println!("series\tp10\tp50\tp90");
    for (label, values) in [("old", &mut old_values), ("new", &mut new_values), ("canonical", &mut canonical_values)] {
        let q = quantiles(values, &[0.1, 0.5, 0.9]);
        println!("{label}\t{:.1}\t{:.1}\t{:.1}", q[0], q[1], q[2]);
    }
    ExitCode::SUCCESS
}
