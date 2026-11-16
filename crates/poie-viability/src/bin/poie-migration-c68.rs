// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase B: C68 deterministic Monte Carlo, legacy vs canonical.
//! LCG a=6364136223846793005 c=1442695040888963407; 20 seeds x 100 runs.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions};
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

const SEEDS: u64 = 20;
const RUNS_PER_SEED: usize = 100;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn next(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
    *state
}

fn stats(horizons: &mut [u32]) -> [f64; 7] {
    horizons.sort_unstable();
    let runs = horizons.len();
    let sum: u32 = horizons.iter().sum();
    let survived = horizons.iter().filter(|days| **days >= 90).count();
    let percentile = |fraction: f64| {
        let index = ((runs - 1) as f64 * fraction).round() as usize;
        f64::from(horizons[index])
    };
    [
        f64::from(horizons[0]),
        percentile(0.10),
        percentile(0.50),
        f64::from(sum) / runs as f64,
        percentile(0.90),
        f64::from(horizons[runs - 1]),
        survived as f64,
    ]
}

fn main() -> ExitCode {
    let model = Model::load(&root().join("scenarios/base-05/model.toml")).unwrap();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    let mut legacy_horizons = Vec::new();
    let mut canonical_horizons = Vec::new();
    for seed_index in 0..SEEDS {
        let mut rng = 20_261_114_u64.wrapping_add(seed_index.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        for _ in 0..RUNS_PER_SEED {
            let interval = 5 + next(&mut rng) % 26;
            let mut events = Vec::new();
            let mut day = interval;
            while day < 90 {
                events.push(FailureEvent { at_s: day as f64 * 86_400.0, asset: "pump-p1".into() });
                day += interval;
            }
            let lathe_day = 20 + next(&mut rng) % 41;
            events.push(FailureEvent { at_s: lathe_day as f64 * 86_400.0, asset: "lathe-1".into() });
            events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
            let legacy = simulate_multi_service_longevity_with_limits(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86_400.0,
                &events,
                battery,
            )
            .unwrap();
            legacy_horizons.push((legacy.survived_s / 86_400.0).round() as u32);
            let options = EngineOptions {
                services: vec!["potable-water", "workshop-operability"],
                horizon_s: 90.0 * 86_400.0,
                failures: events,
                resupplies: Vec::new(),
                resources: vec!["robot-m1".into(), "lathe-1".into()],
                alternative_pool: vec!["robot-m1".into()],
                battery,
                load_kw: 0.0,
            };
            let canonical = run_engine(&model, &options).unwrap();
            canonical_horizons.push((canonical.survived_s / 86_400.0).round() as u32);
        }
    }
    let legacy = stats(&mut legacy_horizons);
    let canonical = stats(&mut canonical_horizons);
    println!("series\tmin\tp10\tmedian\tmean\tp90\tmax\tsurvived_90");
    println!(
        "legacy\t{:.0}\t{:.0}\t{:.0}\t{:.1}\t{:.0}\t{:.0}\t{:.0}",
        legacy[0], legacy[1], legacy[2], legacy[3], legacy[4], legacy[5], legacy[6]
    );
    println!(
        "canonical\t{:.0}\t{:.0}\t{:.0}\t{:.1}\t{:.0}\t{:.0}\t{:.0}",
        canonical[0], canonical[1], canonical[2], canonical[3], canonical[4], canonical[5], canonical[6]
    );
    for (label, values) in [("legacy", &legacy_horizons), ("canonical", &canonical_horizons)] {
        for (low, high) in [(0, 40), (40, 60), (60, 80), (80, 90), (90, 91)] {
            let count = values.iter().filter(|days| **days >= low && **days < high).count();
            println!("bucket\t{label}\t[{low},{high})\t{count}");
        }
        println!("series_name\t{label}");
    }
    let same = legacy.iter().zip(canonical.iter()).all(|(left, right)| (left - right).abs() < 1e-9);
    println!("classification\t{}", if same { "EXPECTED_AGREEMENT" } else { "SEMANTIC_CORRECTION_IN_REPAIR_FAILURE" });
    ExitCode::SUCCESS
}
