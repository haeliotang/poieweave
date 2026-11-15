// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S5 migration batch 2: dense long repair and resource contention.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions, EngineTermination};
use poie_viability::survival::{simulate_multi_service_longevity_event_driven, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn base08() -> Model {
    Model::load(&root().join("scenarios/base-08/model.toml")).unwrap()
}

fn dense() -> (Model, Vec<FailureEvent>) {
    let mut model = base08();
    for lot in &mut model.lots {
        if lot.resource == "sleeve-p1" {
            lot.si = 0.0;
        }
        if lot.resource == "grease" {
            lot.si = 0.15;
        }
    }
    model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap().duration_s =
        Some(5.0 * 86_400.0);
    let events =
        (1..=15).map(|index| FailureEvent { at_s: index as f64 * 2.0 * 86_400.0, asset: "pump-p1".into() }).collect();
    (model, events)
}

fn contention(two_robots: bool) -> (Model, Vec<FailureEvent>) {
    let mut model = base08();
    for process in &mut model.processes {
        if process.id == "replace-sleeve-p1" {
            process.duration_s = Some(3.0 * 3600.0);
        }
        if process.id == "replace-lathe-nut" {
            process.duration_s = Some(2.0 * 3600.0);
        }
    }
    model.assets.get_mut("pump-p1").unwrap().status = poie_ontology::model::AssetStatus::Available;
    model.assets.get_mut("lathe-1").unwrap().status = poie_ontology::model::AssetStatus::Available;
    if two_robots {
        let mut robot = model.assets.get("robot-m1").unwrap().clone();
        robot.id = "robot-m2".into();
        model.assets.insert("robot-m2".into(), robot);
    }
    let events =
        vec![FailureEvent { at_s: 0.0, asset: "pump-p1".into() }, FailureEvent { at_s: 0.0, asset: "lathe-1".into() }];
    (model, events)
}

fn termination(label: &EngineTermination) -> String {
    match label {
        EngineTermination::CompletedHorizon => "completed".into(),
        EngineTermination::Blocked(reason) => format!("blocked:{reason}"),
        EngineTermination::IncompleteAtHorizon(reason) => format!("incomplete:{reason}"),
    }
}

fn main() -> ExitCode {
    println!("scenario\tcanonical_days\tlegacy_days\tdelta_days\tcanonical_termination\tcanonical_events\tlegacy_events\tcanonical_last_h\tlegacy_last_h");
    let (model, events) = dense();
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
    let canonical_days = canonical.survived_s / 86_400.0;
    let legacy_days = legacy.survived_s / 86_400.0;
    let canonical_last_h = canonical
        .trace
        .iter()
        .filter(|event| event.kind != "horizon")
        .map(|event| event.at_s / 3600.0)
        .fold(0.0_f64, f64::max);
    let legacy_last_h = legacy.log.last().map(|event| event.at_s / 3600.0).unwrap_or(0.0);
    println!(
        "dense-long-repair\t{canonical_days:.1}\t{legacy_days:.1}\t{:.1}\t{}\t{}\t{}\t{:.1}\t{:.1}",
        canonical_days - legacy_days,
        termination(&canonical.termination),
        canonical.trace.len(),
        legacy.log.len(),
        canonical_last_h,
        legacy_last_h
    );

    for (name, robots) in [("contention-1-robot", 1usize), ("contention-2-robots", 2usize)] {
        let (model, events) = contention(robots == 2);
        let options = EngineOptions {
            services: vec!["potable-water", "workshop-operability"],
            horizon_s: 24.0 * 3600.0,
            failures: events.clone(),
            resupplies: Vec::new(),
            resources: (1..=robots).map(|index| format!("robot-m{index}")).collect(),
            alternative_pool: (1..=robots).map(|index| format!("robot-m{index}")).collect(),
            battery: None,
            load_kw: 0.0,
        };
        let canonical = run_engine(&model, &options).unwrap();
        let legacy = simulate_multi_service_longevity_event_driven(
            &model,
            &["potable-water", "workshop-operability"],
            24.0 * 3600.0,
            &events,
            None,
            0.0,
        )
        .unwrap();
        let canonical_days = canonical.survived_s / 86_400.0;
        let legacy_days = legacy.survived_s / 86_400.0;
        let canonical_last_h = canonical
            .trace
            .iter()
            .filter(|event| event.kind != "horizon")
            .map(|event| event.at_s / 3600.0)
            .fold(0.0_f64, f64::max);
        let legacy_last_h = legacy.log.last().map(|event| event.at_s / 3600.0).unwrap_or(0.0);
        println!(
            "{name}\t{canonical_days:.3}\t{legacy_days:.3}\t{:.3}\t{}\t{}\t{}\t{canonical_last_h:.1}\t{legacy_last_h:.1}",
            canonical_days - legacy_days,
            termination(&canonical.termination),
            canonical.trace.len(),
            legacy.log.len()
        );
    }
    ExitCode::SUCCESS
}
