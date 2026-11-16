// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase E sample: regen-cost, energy-recharge and recharge-bound.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions, EngineTermination};
use poie_viability::survival::{simulate_multi_service_longevity_with_load, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
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

fn class(legacy: f64, canonical: f64) -> &'static str {
    if (legacy - canonical).abs() < 1e-9 {
        "EXPECTED_AGREEMENT"
    } else if legacy.round() == canonical.round() {
        "SEMANTIC_CORRECTION"
    } else {
        "UNEXPLAINED"
    }
}

fn main() -> ExitCode {
    let base = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("experiment\trow\tlegacy_days\tcanonical_days\tclassification");

    for duration_multiplier in [1.0_f64, 2.0, 4.0] {
        for energy_multiplier in [1.0_f64, 2.0, 100.0] {
            let mut model = base.clone();
            let process = model.processes.iter_mut().find(|process| process.id == "recycle-bronze").unwrap();
            if let Some(duration) = process.duration_s.as_mut() {
                *duration *= duration_multiplier;
            }
            for input in &mut process.inputs {
                if input.resource == "electric-energy" {
                    input.si *= energy_multiplier;
                }
            }
            let legacy = simulate_multi_service_longevity_with_load(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86_400.0,
                &events(),
                battery,
                0.0,
            )
            .unwrap();
            let (canonical_days, _) = canonical(&model, 0.0);
            let legacy_days = legacy.survived_s / 86_400.0;
            println!("regen-cost\tduration={duration_multiplier:.1},energy={energy_multiplier:.1}\t{legacy_days:.0}\t{canonical_days:.3}\t{}", class(legacy_days, canonical_days));
        }
    }

    for kwh in [20.0_f64, 30.0, 40.0] {
        let mut model = base.clone();
        let process = model.processes.iter_mut().find(|process| process.id == "recharge-energy").unwrap();
        for output in &mut process.outputs {
            if output.resource == "electric-energy" {
                output.si = kwh * 3_600_000.0;
            }
        }
        let legacy = simulate_multi_service_longevity_with_load(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &events(),
            battery,
            2.6,
        )
        .unwrap();
        let (canonical_days, _) = canonical(&model, 2.6);
        let legacy_days = legacy.survived_s / 86_400.0;
        println!(
            "energy-recharge\tkwh={kwh:.0}\t{legacy_days:.0}\t{canonical_days:.3}\t{}",
            class(legacy_days, canonical_days)
        );
    }

    let recharge = base.processes.iter().find(|process| process.id == "recharge-energy").unwrap();
    let hours = recharge.duration_s.unwrap() / 3600.0;
    let kwh = recharge.outputs.iter().find(|output| output.resource == "electric-energy").unwrap().si / 3_600_000.0;
    let average_kw = kwh / hours;
    let mut threshold = 0.0;
    for tenths in 20..=40 {
        let load_kw = f64::from(tenths) / 10.0;
        let (_, termination) = canonical(&base, load_kw);
        if termination.contains("energy_depleted") {
            threshold = load_kw;
            break;
        }
    }
    println!(
        "recharge-bound\taverage={average_kw:.2}\tlegacy=2.6\tcanonical={threshold:.1}\t{}",
        if (threshold - 2.6).abs() < 1e-9 { "EXPECTED_AGREEMENT" } else { "UNEXPLAINED" }
    );
    ExitCode::SUCCESS
}
