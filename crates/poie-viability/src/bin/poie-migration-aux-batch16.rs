// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase E sample: grease-cost and yield-accumulation.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{Flow, Model, Process, Verification, YieldSpec};
use poie_viability::engine::{run_engine, EngineOptions};
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn canonical(model: &Model, events: Vec<FailureEvent>, horizon_s: f64) -> f64 {
    let options = EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s,
        failures: events,
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        load_kw: 0.0,
    };
    run_engine(model, &options).unwrap().survived_s / 86_400.0
}

fn sharpen_process(fraction: f64) -> Process {
    Process {
        id: "sharpen-insert".into(),
        inputs: vec![
            Flow { resource: "worn-insert".into(), si: 1.0 },
            Flow { resource: "electric-energy".into(), si: 0.3 * 3_600_000.0 },
        ],
        outputs: vec![Flow { resource: "cutting-insert".into(), si: 1.0 }],
        occupies: vec!["gauge-1".into()],
        requires_capabilities: Vec::new(),
        restores: Vec::new(),
        duration_s: Some(3600.0),
        verification: Verification::Modeled,
        asset_mass_change_kg: 0.0,
        mass_tolerance: 0.01,
        process_yield: if fraction >= 1.0 {
            None
        } else {
            Some(YieldSpec { output: "cutting-insert".into(), fraction, reject: "worn-insert-reject".into() })
        },
    }
}

fn main() -> ExitCode {
    let base = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("experiment\trow\tlegacy_days\tcanonical_days\tclassification");
    let grease_events = vec![
        FailureEvent { at_s: 10.0 * 86_400.0, asset: "pump-p1".into() },
        FailureEvent { at_s: 20.0 * 86_400.0, asset: "pump-p1".into() },
        FailureEvent { at_s: 30.0 * 86_400.0, asset: "pump-p1".into() },
    ];
    for grease_kg in [0.10_f64, 0.15, 0.20] {
        for (mode, grease_per_repair) in [("cheap", 0.05_f64), ("expensive", 0.20)] {
            let mut model = base.clone();
            let lot = model.lots.iter_mut().find(|lot| lot.id == "lot-grease").unwrap();
            lot.si = grease_kg;
            let process = model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap();
            for input in &mut process.inputs {
                if input.resource == "grease" {
                    input.si = grease_per_repair;
                }
            }
            let legacy = simulate_multi_service_longevity_with_limits(
                &model,
                &["potable-water", "workshop-operability"],
                30.0 * 86_400.0,
                &grease_events,
                battery,
            )
            .unwrap();
            let canonical_days = canonical(&model, grease_events.clone(), 30.0 * 86_400.0);
            let legacy_days = legacy.survived_s / 86_400.0;
            let class =
                if legacy_days.round() == canonical_days.round() { "EXPECTED_AGREEMENT" } else { "UNEXPLAINED" };
            println!("grease-cost\tgrease={grease_kg:.2},mode={mode}\t{legacy_days:.0}\t{canonical_days:.3}\t{class}");
        }
    }
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86_400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86_400.0, asset: "lathe-1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    for fraction in [1.0_f64, 0.9, 0.75, 0.5] {
        let mut model = base.clone();
        model.add_process(sharpen_process(fraction));
        if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "cutting-insert") {
            lot.si = 2.0;
        }
        if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "bronze-bar") {
            lot.si = 3.0;
        }
        let legacy = simulate_multi_service_longevity_with_limits(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &events,
            battery,
        )
        .unwrap();
        let canonical_days = canonical(&model, events.clone(), 90.0 * 86_400.0);
        let legacy_days = legacy.survived_s / 86_400.0;
        let class = if legacy_days.round() == canonical_days.round() { "EXPECTED_AGREEMENT" } else { "UNEXPLAINED" };
        println!("yield-accumulation\tyield={fraction:.2}\t{legacy_days:.0}\t{canonical_days:.3}\t{class}");
    }
    ExitCode::SUCCESS
}
