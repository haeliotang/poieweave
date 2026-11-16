// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase B: C12 horizon/insert/repair-sweep components under canonical semantics.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions, EngineTermination};
use poie_viability::survival::{
    simulate_multi_service_longevity_event_driven, simulate_multi_service_longevity_with_load, BatteryLimits,
    FailureEvent,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn events() -> Vec<FailureEvent> {
    (1..=15).map(|index| FailureEvent { at_s: f64::from(index * 2) * 86_400.0, asset: "pump-p1".into() }).collect()
}

fn canonical(model: &Model, horizon_s: f64) -> (f64, String) {
    let options = EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s,
        failures: events(),
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        load_kw: 0.0,
    };
    let outcome = run_engine(model, &options).unwrap();
    let termination = match &outcome.termination {
        EngineTermination::CompletedHorizon => "completed".to_string(),
        EngineTermination::Blocked(reason) => format!("blocked:{reason}"),
        EngineTermination::IncompleteAtHorizon(reason) => format!("incomplete:{reason}"),
    };
    (outcome.survived_s / 86_400.0, termination)
}

fn classify(legacy: f64, canonical: f64) -> &'static str {
    if (legacy - canonical).abs() < 1e-9 {
        "EXPECTED_AGREEMENT"
    } else if legacy.round() != canonical.round() && canonical < legacy {
        "SEMANTIC_CORRECTION_IN_REPAIR_FAILURE"
    } else {
        "SEMANTIC_CORRECTION"
    }
}

fn main() -> ExitCode {
    let base = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });

    let mut horizon_model = base.clone();
    for lot in &mut horizon_model.lots {
        if lot.resource == "sleeve-p1" {
            lot.si = 0.0;
        }
        if lot.resource == "grease" {
            lot.si = 0.15;
        }
    }
    horizon_model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap().duration_s =
        Some(5.0 * 86_400.0);
    let instant = simulate_multi_service_longevity_with_load(
        &horizon_model,
        &["potable-water", "workshop-operability"],
        30.0 * 86_400.0,
        &events(),
        battery,
        0.0,
    )
    .unwrap();
    let event = simulate_multi_service_longevity_event_driven(
        &horizon_model,
        &["potable-water", "workshop-operability"],
        30.0 * 86_400.0,
        &events(),
        battery,
        0.0,
    )
    .unwrap();
    let (canonical_days, termination) = canonical(&horizon_model, 30.0 * 86_400.0);
    println!("component\tlegacy_instant\tlegacy_event\tcanonical\tclassification");
    println!(
        "horizon\t{:.0}\t{:.0}\t{canonical_days:.3}\t{}\t{termination}",
        instant.survived_s / 86_400.0,
        event.survived_s / 86_400.0,
        classify(event.survived_s / 86_400.0, canonical_days)
    );

    println!("inserts\tlegacy_instant\tlegacy_event\tcanonical\tclassification");
    for inserts in [1.0_f64, 2.0, 3.0, 4.0, 6.0] {
        let mut model = base.clone();
        model.lots.iter_mut().find(|lot| lot.id == "lot-insert").unwrap().si = inserts;
        model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap().duration_s =
            Some(5.0 * 86_400.0);
        let old = simulate_multi_service_longevity_with_load(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &events(),
            battery,
            0.0,
        )
        .unwrap();
        let new = simulate_multi_service_longevity_event_driven(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &events(),
            battery,
            0.0,
        )
        .unwrap();
        let (canonical_days, _) = canonical(&model, 90.0 * 86_400.0);
        println!(
            "{inserts:.0}\t{:.0}\t{:.0}\t{canonical_days:.3}\t{}",
            old.survived_s / 86_400.0,
            new.survived_s / 86_400.0,
            classify(new.survived_s / 86_400.0, canonical_days)
        );
    }

    println!("repair_days\tlegacy_instant\tlegacy_event\tcanonical\tclassification");
    for repair_days in [0.125_f64, 0.5, 1.0, 2.0, 3.0, 5.0] {
        let mut model = base.clone();
        for lot in &mut model.lots {
            if lot.resource == "sleeve-p1" {
                lot.si = 0.0;
            }
            if lot.resource == "grease" {
                lot.si = 0.15;
            }
        }
        model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap().duration_s =
            Some(repair_days * 86_400.0);
        let old = simulate_multi_service_longevity_with_load(
            &model,
            &["potable-water", "workshop-operability"],
            30.0 * 86_400.0,
            &events(),
            battery,
            0.0,
        )
        .unwrap();
        let new = simulate_multi_service_longevity_event_driven(
            &model,
            &["potable-water", "workshop-operability"],
            30.0 * 86_400.0,
            &events(),
            battery,
            0.0,
        )
        .unwrap();
        let (canonical_days, _) = canonical(&model, 30.0 * 86_400.0);
        println!(
            "{repair_days}\t{:.0}\t{:.0}\t{canonical_days:.3}\t{}",
            old.survived_s / 86_400.0,
            new.survived_s / 86_400.0,
            classify(new.survived_s / 86_400.0, canonical_days)
        );
    }
    ExitCode::SUCCESS
}
