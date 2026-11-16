// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase E sample: joint-boundary and joint-consumables.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{Flow, Model, Process, Verification};
use poie_viability::engine::{run_engine, EngineOptions, EngineTermination};
use poie_viability::survival::{
    simulate_multi_service_longevity_with_limits, simulate_multi_service_longevity_with_load, BatteryLimits,
    FailureEvent,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sharpen_process() -> Process {
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
        process_yield: None,
    }
}

fn events() -> Vec<FailureEvent> {
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86_400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86_400.0, asset: "lathe-1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    events
}

fn canonical(model: &Model, load_kw: f64) -> (f64, String) {
    let options = EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s: 90.0 * 86_400.0,
        failures: events(),
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        load_kw,
    };
    let outcome = run_engine(model, &options).unwrap();
    let termination = match &outcome.termination {
        EngineTermination::CompletedHorizon => "completed".to_string(),
        EngineTermination::Blocked(reason) => format!("blocked:{reason}"),
        EngineTermination::IncompleteAtHorizon(reason) => format!("incomplete:{reason}"),
    };
    (outcome.survived_s / 86_400.0, termination)
}

fn main() -> ExitCode {
    let base = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("experiment\trow\tlegacy_days\tcanonical_days\tclassification");
    for load_kw in [2.4_f64, 2.5, 2.6] {
        for bronze in [3.0_f64, 6.0] {
            let mut model = base.clone();
            model.add_process(sharpen_process());
            if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "cutting-insert") {
                lot.si = 2.0;
            }
            if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "bronze-bar") {
                lot.si = bronze;
            }
            let legacy = simulate_multi_service_longevity_with_load(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86_400.0,
                &events(),
                battery,
                load_kw,
            )
            .unwrap();
            let (canonical_days, _) = canonical(&model, load_kw);
            let legacy_days = legacy.survived_s / 86_400.0;
            let class = if (legacy_days - canonical_days).abs() < 1e-9 {
                "EXPECTED_AGREEMENT"
            } else if legacy_days.round() == canonical_days.round() {
                "SEMANTIC_CORRECTION"
            } else {
                "UNEXPLAINED"
            };
            println!(
                "joint-boundary\tload={load_kw:.1},bronze={bronze:.1}\t{legacy_days:.0}\t{canonical_days:.3}\t{class}"
            );
        }
    }
    for inserts in [2.0_f64, 4.0] {
        for bronze in [0.0_f64, 1.5, 3.0, 6.0] {
            let mut model = base.clone();
            model.add_process(sharpen_process());
            if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "cutting-insert") {
                lot.si = inserts;
            }
            if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "bronze-bar") {
                lot.si = bronze;
            }
            let legacy = simulate_multi_service_longevity_with_limits(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86_400.0,
                &events(),
                battery,
            )
            .unwrap();
            let (canonical_days, _) = canonical(&model, 0.0);
            let legacy_days = legacy.survived_s / 86_400.0;
            let class = if (legacy_days - canonical_days).abs() < 1e-9 {
                "EXPECTED_AGREEMENT"
            } else if legacy_days.round() == canonical_days.round() {
                "SEMANTIC_CORRECTION"
            } else {
                "UNEXPLAINED"
            };
            println!("joint-consumables\tinserts={inserts:.0},bronze={bronze:.1}\t{legacy_days:.0}\t{canonical_days:.3}\t{class}");
        }
    }
    ExitCode::SUCCESS
}
