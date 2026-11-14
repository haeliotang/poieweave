// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 random adversarial sample: random subsets and failure times.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
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

fn run(model: &Model, failed: &[&str], at_s: f64) -> f64 {
    let events: Vec<FailureEvent> = failed.iter().map(|asset| FailureEvent { at_s, asset: (*asset).into() }).collect();
    simulate_multi_service_longevity_with_limits(
        model,
        &["potable-water", "workshop-operability"],
        90.0 * 86400.0,
        &events,
        Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
    )
    .unwrap()
    .survived_s
        / 86400.0
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let assets = ["pump-p1", "filter-1", "robot-m1", "pv-1", "lathe-1"];
    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    let trials = 500usize;
    let mut zero = 0usize;
    let mut under30 = 0usize;
    let mut under60 = 0usize;
    let mut total = 0.0;
    let mut best = (f64::INFINITY, usize::MAX, String::new(), 0.0);
    for _ in 0..trials {
        let mask = 1 + (next(&mut state) % 31) as u32;
        let day = (next(&mut state) % 91) as f64;
        let failed: Vec<&str> =
            (0..assets.len()).filter(|index| mask & (1 << index) != 0).map(|index| assets[index]).collect();
        let days = run(&model, &failed, day * 86_400.0);
        total += days;
        if days == 0.0 {
            zero += 1;
        }
        if days < 30.0 {
            under30 += 1;
        }
        if days < 60.0 {
            under60 += 1;
        }
        if days < best.0 || (days == best.0 && failed.len() < best.1) {
            best = (days, failed.len(), failed.join("+"), day);
        }
    }
    println!("trials\tzero\tunder30\tunder60\tmean_days\tmin_days\tworst_set\tat_day");
    println!(
        "{trials}\t{zero}\t{under30}\t{under60}\t{:.1}\t{:.0}\t{}\t{:.0}",
        total / trials as f64,
        best.0,
        best.2,
        best.3
    );
    ExitCode::SUCCESS
}
