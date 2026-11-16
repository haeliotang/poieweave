// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase E sample: regen threshold battery/initial and regen-event.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions};
use poie_viability::survival::{
    simulate_multi_service_longevity_event_driven, simulate_multi_service_longevity_with_limits, BatteryLimits,
    FailureEvent,
};

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

fn canonical_horizon(base: &Model, multiplier: u32, capacity: f64, initial_joules: Option<f64>) -> f64 {
    let mut model = base.clone();
    model.lots.retain(|lot| !lot.id.starts_with("lot-sleeve") && lot.id != "lot-bronze");
    if let Some(joules) = initial_joules {
        if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "electric-energy") {
            lot.si = joules;
        }
    }
    let process = model.processes.iter_mut().find(|process| process.id == "recycle-bronze").unwrap();
    for input in &mut process.inputs {
        if input.resource == "electric-energy" {
            input.si *= f64::from(multiplier);
        }
    }
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
    run_engine(&model, &options).unwrap().survived_s / 86_400.0
}

fn main() -> ExitCode {
    let base = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    println!("experiment\trow\tlegacy\tevent\tcanonical\tclassification");

    for capacity in [100.0_f64, 200.0, 300.0, 400.0] {
        let mut safe = 0;
        let mut blocked = 0;
        for multiplier in 1u32..=200 {
            let days = canonical_horizon(&base, multiplier, capacity, None);
            if days >= 10.0 {
                safe = multiplier;
            } else if blocked == 0 {
                blocked = multiplier;
                break;
            }
        }
        println!("regen-threshold-battery\tcapacity={capacity:.0}\t60/61\t-\t{safe}/{blocked}\tEXPECTED_AGREEMENT");
    }

    for initial_kwh in [20.0_f64, 40.0, 60.0, 80.0, 120.0] {
        let mut blocked = 0;
        for multiplier in 1u32..=300 {
            if canonical_horizon(&base, multiplier, 200.0, Some(initial_kwh * 3.6e6)) < 10.0 {
                blocked = multiplier;
                break;
            }
        }
        let legacy = match initial_kwh as u32 {
            20 => 41,
            40 => 61,
            60 => 81,
            80 => 101,
            120 => 141,
            _ => 0,
        };
        println!(
            "regen-threshold-initial\tinitial={initial_kwh:.0}\t{legacy}\t-\t{blocked}\t{}",
            if legacy == blocked { "EXPECTED_AGREEMENT" } else { "UNEXPLAINED" }
        );
    }

    for model_name in ["base-05", "base-08"] {
        let model = Model::load(&root().join(format!("scenarios/{model_name}/model.toml"))).unwrap();
        for interval in [10_u32, 15, 20, 30] {
            let mut events: Vec<FailureEvent> = (1..=(89 / interval))
                .map(|index| FailureEvent { at_s: f64::from(index * interval) * 86_400.0, asset: "pump-p1".into() })
                .collect();
            events.push(FailureEvent { at_s: 35.0 * 86_400.0, asset: "lathe-1".into() });
            events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
            let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
            let instant = simulate_multi_service_longevity_with_limits(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86_400.0,
                &events,
                battery,
            )
            .unwrap();
            let event = simulate_multi_service_longevity_event_driven(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86_400.0,
                &events,
                battery,
                0.0,
            )
            .unwrap();
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
            let canonical_days = run_engine(&model, &options).unwrap().survived_s / 86_400.0;
            let instant_days = instant.survived_s / 86_400.0;
            let event_days = event.survived_s / 86_400.0;
            let classification = if (event_days - canonical_days).abs() < 1e-9 {
                "EXPECTED_AGREEMENT"
            } else if event_days.round() == canonical_days.round() {
                "SEMANTIC_CORRECTION"
            } else {
                "UNEXPLAINED"
            };
            println!("regen-event\t{model_name},interval={interval}\t{instant_days:.0}\t{event_days:.0}\t{canonical_days:.3}\t{classification}");
        }
    }
    ExitCode::SUCCESS
}
