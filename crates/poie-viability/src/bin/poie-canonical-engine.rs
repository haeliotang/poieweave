// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S5: canonical kernel vs a legacy survival entry point.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions, EngineTermination};
use poie_viability::survival::{simulate_multi_service_longevity_event_driven, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let mut model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    for lot in &mut model.lots {
        if lot.resource == "sleeve-p1" {
            lot.si = 10.0;
        }
    }
    model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap().duration_s =
        Some(5.0 * 86_400.0);
    let events = vec![
        FailureEvent { at_s: 10.0 * 86_400.0, asset: "pump-p1".into() },
        FailureEvent { at_s: 20.0 * 86_400.0, asset: "pump-p1".into() },
    ];
    let options = EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s: 30.0 * 86_400.0,
        failures: events.clone(),
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        load_kw: 0.0,
    };
    let canonical = run_engine(&model, &options).unwrap();
    let legacy = simulate_multi_service_longevity_event_driven(
        &model,
        &["potable-water", "workshop-operability"],
        30.0 * 86_400.0,
        &events,
        Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        0.0,
    )
    .unwrap();
    let verdict = if canonical.survived_s == legacy.survived_s { "same" } else { "different" };
    println!("scenario\tcanonical_days\tlegacy_days\ttermination\tverdict");
    let termination = match &canonical.termination {
        EngineTermination::CompletedHorizon => "completed".to_string(),
        EngineTermination::Blocked(reason) => format!("blocked:{reason}"),
        EngineTermination::IncompleteAtHorizon(reason) => format!("incomplete:{reason}"),
    };
    println!(
        "base-08-sparse\t{:.0}\t{:.0}\t{}\t{}",
        canonical.survived_s / 86_400.0,
        legacy.survived_s / 86_400.0,
        termination,
        verdict
    );
    ExitCode::SUCCESS
}
