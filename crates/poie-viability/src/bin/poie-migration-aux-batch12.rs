// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase E sample: regen-forced-cost and regen-capacity-semantics.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions};
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

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

fn forced_model(base: &Model, multiplier: f64) -> Model {
    let mut model = base.clone();
    model.lots.retain(|lot| !lot.id.starts_with("lot-sleeve") && lot.id != "lot-bronze");
    let process = model.processes.iter_mut().find(|process| process.id == "recycle-bronze").unwrap();
    for input in &mut process.inputs {
        if input.resource == "electric-energy" {
            input.si *= multiplier;
        }
    }
    model
}

fn capacity_model(base: &Model, multiplier: u32, capacity: f64, initial_joules: f64) -> Model {
    let mut model = forced_model(base, f64::from(multiplier));
    if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "electric-energy") {
        lot.si = initial_joules;
    }
    let _ = capacity;
    model
}

fn canonical_horizon(model: &Model, capacity: f64) -> Result<f64, String> {
    let options = EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s: 90.0 * 86_400.0,
        failures: events(),
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(BatteryLimits { capacity_kwh: capacity, charge_kw: 10.0 }),
        load_kw: 0.0,
    };
    run_engine(model, &options).map(|outcome| outcome.survived_s / 86_400.0).map_err(|error| error.0)
}

fn main() -> ExitCode {
    let base = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("experiment\trow\tlegacy\tcanonical\tclassification");
    for multiplier in [1.0_f64, 10.0, 50.0, 100.0] {
        let model = forced_model(&base, multiplier);
        let legacy = simulate_multi_service_longevity_with_limits(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &events(),
            battery,
        )
        .unwrap();
        let legacy_days = legacy.survived_s / 86_400.0;
        let canonical = canonical_horizon(&model, 200.0).unwrap_or(0.0);
        let class = if legacy_days.round() == canonical.round() { "EXPECTED_AGREEMENT" } else { "UNEXPLAINED" };
        println!("regen-forced-cost\tenergy_x={multiplier:.0}\t{legacy_days:.0}\t{canonical:.3}\t{class}");
    }
    for capacity in [200.0_f64, 400.0, 500.0, 600.0] {
        let mut legacy_result = Ok(0);
        for multiplier in 1u32..=700 {
            let model = capacity_model(&base, multiplier, capacity, 500.0 * 3.6e6);
            match simulate_multi_service_longevity_with_limits(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86_400.0,
                &events(),
                Some(BatteryLimits { capacity_kwh: capacity, charge_kw: 10.0 }),
            ) {
                Ok(report) if report.survived_s / 86_400.0 < 10.0 => {
                    legacy_result = Ok(multiplier);
                    break;
                }
                Ok(_) => {}
                Err(error) => {
                    legacy_result = Err(error.to_string());
                    break;
                }
            }
        }
        let mut canonical_result = Ok(0);
        for multiplier in 1u32..=700 {
            let model = capacity_model(&base, multiplier, capacity, 500.0 * 3.6e6);
            match canonical_horizon(&model, capacity) {
                Ok(days) if days < 10.0 => {
                    canonical_result = Ok(multiplier);
                    break;
                }
                Ok(_) => {}
                Err(error) => {
                    canonical_result = Err(error);
                    break;
                }
            }
        }
        let legacy_text = match legacy_result {
            Ok(0) => "rejected".into(),
            Ok(value) => value.to_string(),
            Err(_) => "rejected".into(),
        };
        let canonical_text = match canonical_result {
            Ok(0) => "rejected".into(),
            Ok(value) => value.to_string(),
            Err(_) => "rejected".into(),
        };
        let class = if legacy_text == canonical_text { "EXPECTED_AGREEMENT" } else { "UNEXPLAINED" };
        println!("regen-capacity-semantics\tcapacity={capacity:.0}\t{legacy_text}\t{canonical_text}\t{class}");
    }
    ExitCode::SUCCESS
}
