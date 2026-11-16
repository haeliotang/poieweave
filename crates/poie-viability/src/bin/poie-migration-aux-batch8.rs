// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase E sample: regen-long and joint double-failure.

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

fn main() -> ExitCode {
    println!("experiment\trow\tlegacy_instant\tlegacy_event\tcanonical\tclassification");
    for model_name in ["base-05", "base-08"] {
        let mut model = Model::load(&root().join(format!("scenarios/{model_name}/model.toml"))).unwrap();
        model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap().duration_s =
            Some(5.0 * 86_400.0);
        for interval in [2_u32, 3, 5, 10] {
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
                "SEMANTIC_CORRECTION_IN_REPAIR_FAILURE"
            };
            println!("regen-long\t{model_name},interval={interval}\t{instant_days:.0}\t{event_days:.0}\t{canonical_days:.3}\t{classification}");
        }
    }

    let model = Model::load(&root().join("scenarios/base-06/model.toml")).unwrap();
    let events = vec![
        FailureEvent { at_s: 10.0 * 86_400.0, asset: "pump-p1".into() },
        FailureEvent { at_s: 10.0 * 86_400.0, asset: "pump-p2".into() },
    ];
    let legacy = simulate_multi_service_longevity_with_limits(
        &model,
        &["potable-water"],
        30.0 * 86_400.0,
        &events,
        Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
    )
    .unwrap();
    let options = EngineOptions {
        services: vec!["potable-water"],
        horizon_s: 30.0 * 86_400.0,
        failures: events,
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        load_kw: 0.0,
    };
    let canonical_days = run_engine(&model, &options).unwrap().survived_s / 86_400.0;
    let legacy_days = legacy.survived_s / 86_400.0;
    println!(
        "joint-double-failure\tpump-p1,pump-p2\t{legacy_days:.0}\t-\t{canonical_days:.3}\t{}",
        if (legacy_days - canonical_days).abs() < 1e-9 {
            "EXPECTED_AGREEMENT"
        } else if legacy_days.round() == canonical_days.round() {
            "SEMANTIC_CORRECTION_REPAIR_SCHEDULE"
        } else {
            "UNEXPLAINED"
        }
    );
    ExitCode::SUCCESS
}
