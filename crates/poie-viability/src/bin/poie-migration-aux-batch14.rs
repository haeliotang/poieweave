// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase E sample: adversarial-time, regeneration-horizon and regeneration-yield.

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

fn main() -> ExitCode {
    let mut model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap().duration_s =
        Some(5.0 * 86_400.0);
    let assets = ["pump-p1", "filter-1", "robot-m1", "pv-1", "lathe-1"];
    println!("experiment\trow\tlegacy_instant\tlegacy_event\tcanonical\tclassification");
    for at_day in [0.0_f64, 10.0] {
        let mut min_old = f64::INFINITY;
        let mut min_new = f64::INFINITY;
        let mut min_canonical = f64::INFINITY;
        let mut min_set = String::new();
        for mask in 1u32..(1 << assets.len()) {
            let failed: Vec<&str> =
                (0..assets.len()).filter(|index| mask & (1 << index) != 0).map(|index| assets[index]).collect();
            let mut events: Vec<FailureEvent> = (1..=45)
                .map(|index| FailureEvent { at_s: f64::from(index * 2) * 86_400.0, asset: "pump-p1".into() })
                .collect();
            events.push(FailureEvent { at_s: 35.0 * 86_400.0, asset: "lathe-1".into() });
            events.extend(failed.iter().map(|asset| FailureEvent { at_s: 0.0, asset: (*asset).into() }));
            let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
            let old = simulate_multi_service_longevity_with_limits(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86_400.0,
                &events,
                battery,
            )
            .unwrap();
            let new = simulate_multi_service_longevity_event_driven(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86_400.0,
                &events,
                battery,
                0.0,
            )
            .unwrap();
            let canonical_events: Vec<FailureEvent> = (1..=45)
                .map(|index| FailureEvent { at_s: f64::from(index * 2) * 86_400.0, asset: "pump-p1".into() })
                .chain(std::iter::once(FailureEvent { at_s: 35.0 * 86_400.0, asset: "lathe-1".into() }))
                .chain(failed.iter().map(|asset| FailureEvent { at_s: at_day * 86_400.0, asset: (*asset).into() }))
                .collect();
            let canonical_days = canonical(&model, canonical_events, 90.0 * 86_400.0);
            min_old = min_old.min(old.survived_s / 86_400.0);
            min_new = min_new.min(new.survived_s / 86_400.0);
            if canonical_days < min_canonical {
                min_canonical = canonical_days;
                min_set = failed.join("+");
            }
        }
        let class = if min_new.round() == min_canonical.round() {
            "EXPECTED_AGREEMENT"
        } else {
            "SEMANTIC_CORRECTION_LEGACY_IGNORED_AT_DAY"
        };
        println!("adversarial-time\tday={at_day:.0},min_set={min_set}\t{min_old:.0}\t{min_new:.0}\t{min_canonical:.3}\t{class}");
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
            let legacy = simulate_multi_service_longevity_with_limits(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86_400.0,
                &events,
                battery,
            )
            .unwrap();
            let canonical_days = canonical(&model, events, 90.0 * 86_400.0);
            let legacy_days = legacy.survived_s / 86_400.0;
            let class = if (legacy_days - canonical_days).abs() < 1e-9 {
                "EXPECTED_AGREEMENT"
            } else if legacy_days.round() == canonical_days.round() {
                "SEMANTIC_CORRECTION"
            } else {
                "UNEXPLAINED"
            };
            println!("regeneration-horizon\t{model_name},interval={interval}\t{legacy_days:.0}\t-\t{canonical_days:.3}\t{class}");
        }
    }

    let base = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    for fraction in [1.0_f64, 0.75, 0.5] {
        let mut model = base.clone();
        let process = model.processes.iter_mut().find(|process| process.id == "recycle-bronze").unwrap();
        for output in &mut process.outputs {
            if output.resource == "bronze-bar" {
                output.si *= fraction;
            }
        }
        let mut events: Vec<FailureEvent> = (1..=8)
            .map(|index| FailureEvent { at_s: f64::from(index * 10) * 86_400.0, asset: "pump-p1".into() })
            .collect();
        events.push(FailureEvent { at_s: 35.0 * 86_400.0, asset: "lathe-1".into() });
        events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
        let legacy = simulate_multi_service_longevity_with_limits(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &events,
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        )
        .unwrap();
        let canonical_days = canonical(&model, events, 90.0 * 86_400.0);
        let legacy_days = legacy.survived_s / 86_400.0;
        let class = if legacy_days.round() == canonical_days.round() { "EXPECTED_AGREEMENT" } else { "UNEXPLAINED" };
        println!("regeneration-yield\tyield={fraction:.2}\t{legacy_days:.0}\t-\t{canonical_days:.3}\t{class}");
    }
    ExitCode::SUCCESS
}
