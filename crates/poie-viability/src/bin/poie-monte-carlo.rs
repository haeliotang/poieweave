// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 deterministic Monte Carlo: random pump/lathe failure schedules on Base-05.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn next(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
    *state
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-05/model.toml")).unwrap_or_else(|error| {
        panic!("load Base-05: {error}");
    });
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    const SEEDS: u64 = 20;
    const RUNS_PER_SEED: usize = 100;
    let mut horizons = Vec::new();
    for seed_index in 0..SEEDS {
        let mut rng = 20_261_114_u64.wrapping_add(seed_index.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        for _ in 0..RUNS_PER_SEED {
            let interval = 5 + next(&mut rng) % 26; // 5..=30 days
            let mut events = Vec::new();
            let mut day = interval;
            while day < 90 {
                events.push(FailureEvent { at_s: day as f64 * 86400.0, asset: "pump-p1".into() });
                day += interval;
            }
            let lathe_day = 20 + next(&mut rng) % 41; // 20..=60
            events.push(FailureEvent { at_s: lathe_day as f64 * 86400.0, asset: "lathe-1".into() });
            events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
            let report = simulate_multi_service_longevity_with_limits(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86400.0,
                &events,
                battery,
            )
            .unwrap();
            horizons.push((report.survived_s / 86400.0).round() as u32);
        }
    }
    horizons.sort_unstable();
    let runs = horizons.len();
    let sum: u32 = horizons.iter().sum();
    let survived = horizons.iter().filter(|days| **days >= 90).count();
    let percentile = |fraction: f64| {
        let index = ((runs - 1) as f64 * fraction).round() as usize;
        horizons[index]
    };
    println!(
        "runs={runs} seeds={SEEDS} min_days={} p10_days={} median_days={} mean_days={:.1} p90_days={} max_days={} survived_90={survived}",
        horizons[0],
        percentile(0.10),
        percentile(0.50),
        f64::from(sum) / runs as f64,
        percentile(0.90),
        horizons[runs - 1]
    );
    for bucket in [(0, 40), (40, 60), (60, 80), (80, 90), (90, 91)] {
        let count = horizons.iter().filter(|days| **days >= bucket.0 && **days < bucket.1).count();
        if count > 0 {
            println!("bucket\t[{},{})\t{count}", bucket.0, bucket.1);
        }
    }
    ExitCode::SUCCESS
}
