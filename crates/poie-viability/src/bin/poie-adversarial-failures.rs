// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 adversarial search: minimal simultaneous failure sets that shorten survival.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run(model: &Model, failed: &[&str]) -> (f64, String) {
    let events: Vec<FailureEvent> =
        failed.iter().map(|asset| FailureEvent { at_s: 0.0, asset: (*asset).into() }).collect();
    let report = simulate_multi_service_longevity_with_limits(
        model,
        &["potable-water", "workshop-operability"],
        60.0 * 86400.0,
        &events,
        Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
    )
    .unwrap();
    let blocked = report.log.iter().find_map(|entry| entry.blocked_by.clone()).unwrap_or_else(|| "none".into());
    (report.survived_s / 86400.0, blocked)
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let assets = ["pump-p1", "filter-1", "robot-m1", "pv-1", "lathe-1"];
    println!("set\tsize\tsurvived_days\tblocked_by");
    let (base_days, base_blocked) = run(&model, &[]);
    println!("baseline\t0\t{base_days:.0}\t{base_blocked}");
    let mut worst = (base_days, 0usize, String::from("baseline"));
    for (index, asset) in assets.iter().enumerate() {
        let (days, blocked) = run(&model, &[*asset]);
        println!("{asset}\t1\t{days:.0}\t{blocked}");
        if days < worst.0 || (days == worst.0 && 1 < worst.1) {
            worst = (days, 1, (*asset).to_string());
        }
        for other in &assets[index + 1..] {
            let (days, blocked) = run(&model, &[*asset, *other]);
            println!("{asset}+{other}\t2\t{days:.0}\t{blocked}");
            if days < worst.0 || (days == worst.0 && 2 < worst.1) {
                worst = (days, 2, format!("{asset}+{other}"));
            }
        }
    }
    println!("worst\t{}\t{:.0}\t{}", worst.1, worst.0, worst.2);
    ExitCode::SUCCESS
}
