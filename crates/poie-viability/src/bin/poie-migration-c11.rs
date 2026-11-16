// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase B: C11 adversarial subsets with/without resupply, legacy vs canonical.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions};
use poie_viability::survival::{
    simulate_multi_service_longevity_with_limits, simulate_multi_service_longevity_with_resupply, BatteryLimits,
    FailureEvent, Resupply,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn events(failed: &[&str], at_day: f64) -> Vec<FailureEvent> {
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86_400.0, asset: "pump-p1".into() }).collect();
    events.extend(failed.iter().map(|asset| FailureEvent { at_s: at_day * 86_400.0, asset: (*asset).into() }));
    events
}

fn supplies() -> Vec<Resupply> {
    (1..=8)
        .map(|index| Resupply { at_s: f64::from(index * 10) * 86_400.0, resource: "cutting-insert".into(), si: 1.0 })
        .collect()
}

fn canonical(model: &Model, failed: &[&str], at_day: f64, with_supply: bool) -> f64 {
    let options = EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s: 90.0 * 86_400.0,
        failures: events(failed, at_day),
        resupplies: if with_supply { supplies() } else { Vec::new() },
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        load_kw: 0.0,
    };
    run_engine(model, &options).unwrap().survived_s / 86_400.0
}

fn main() -> ExitCode {
    let mut model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    model.lots.iter_mut().find(|lot| lot.id == "lot-insert").unwrap().si = 1.0;
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    let assets = ["pump-p1", "filter-1", "robot-m1", "pv-1", "lathe-1"];
    println!("at_day\tlegacy_no_min\tlegacy_yes_min\tcanonical_no_min\tcanonical_yes_min\tlegacy_minimal\tcanonical_minimal\tclassification");
    for at_day in [0.0, 10.0] {
        let mut legacy_rows: Vec<(String, f64, f64)> = Vec::new();
        let mut canonical_rows: Vec<(String, f64, f64)> = Vec::new();
        for mask in 1u32..(1 << assets.len()) {
            let failed: Vec<&str> =
                (0..assets.len()).filter(|index| mask & (1 << index) != 0).map(|index| assets[index]).collect();
            let label = failed.join("+");
            let e0 = events(&failed, 0.0);
            let no = simulate_multi_service_longevity_with_limits(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86_400.0,
                &e0,
                battery,
            )
            .unwrap()
            .survived_s
                / 86_400.0;
            let yes = simulate_multi_service_longevity_with_resupply(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86_400.0,
                &e0,
                &supplies(),
                battery,
                0.0,
            )
            .unwrap()
            .survived_s
                / 86_400.0;
            legacy_rows.push((label.clone(), no, yes));
            canonical_rows.push((
                label,
                canonical(&model, &failed, at_day, false),
                canonical(&model, &failed, at_day, true),
            ));
        }
        let min = |rows: &[(String, f64, f64)], index: usize| {
            rows.iter().map(|row| if index == 1 { row.1 } else { row.2 }).fold(f64::INFINITY, f64::min)
        };
        let pick = |rows: &[(String, f64, f64)], value: f64| {
            let mut candidates: Vec<&(String, f64, f64)> =
                rows.iter().filter(|row| (row.2 - value).abs() < 1e-9).collect();
            candidates.sort_by_key(|row| row.0.len());
            candidates.first().map(|row| row.0.clone()).unwrap_or_default()
        };
        let legacy_no = min(&legacy_rows, 1);
        let legacy_yes = min(&legacy_rows, 2);
        let canonical_no = min(&canonical_rows, 1);
        let canonical_yes = min(&canonical_rows, 2);
        let legacy_set = pick(&legacy_rows, legacy_yes);
        let canonical_set = pick(&canonical_rows, canonical_yes);
        let minima_agree = (legacy_no - canonical_no).abs() < 1e-9 && (legacy_yes - canonical_yes).abs() < 1e-9;
        let classification = if minima_agree && legacy_set == canonical_set {
            "EXPECTED_AGREEMENT"
        } else if minima_agree {
            "EXPECTED_MINIMA_ONLY"
        } else if at_day > 0.0 {
            "SEMANTIC_CORRECTION_LEGACY_IGNORED_AT_DAY"
        } else {
            "UNEXPLAINED"
        };
        println!(
            "{at_day:.0}\t{legacy_no:.0}\t{legacy_yes:.0}\t{canonical_no:.0}\t{canonical_yes:.0}\t{legacy_set}\t{canonical_set}\t{classification}"
        );
    }
    ExitCode::SUCCESS
}
