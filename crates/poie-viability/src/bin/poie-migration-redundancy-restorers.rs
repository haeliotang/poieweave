// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase D: redundancy filter/robot restorer rows under canonical semantics.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{Asset, AssetStatus, Flow, Lot, LotQuality, Model, Process, Resource, Verification};
use poie_ontology::units::UNITS;
use poie_viability::engine::{run_engine, EngineOptions, EngineTermination};
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn redundant_pump(model: &mut Model) {
    model.assets.insert(
        "pump-p2".into(),
        Asset {
            id: "pump-p2".into(),
            kind: "pump".into(),
            status: AssetStatus::Available,
            status_evidence: None,
            requires: Vec::new(),
            requires_any: Vec::new(),
            provides: Vec::new(),
            capability_verification: BTreeMap::new(),
            degraded: None,
        },
    );
    let service = model.services.get_mut("potable-water").unwrap();
    service.requires.retain(|asset| asset != "pump-p1");
    service.requires_any.push("pump-p1".into());
    service.requires_any.push("pump-p2".into());
}

fn add_filter_restorer(model: &mut Model) {
    model.resources.insert(
        "filter-cartridge".into(),
        Resource {
            id: "filter-cartridge".into(),
            unit: &UNITS[0],
            mass_per_unit_kg: Some(1.0),
            quality: LotQuality::Accepted,
        },
    );
    model.resources.insert(
        "failed-filter".into(),
        Resource {
            id: "failed-filter".into(),
            unit: &UNITS[0],
            mass_per_unit_kg: Some(1.0),
            quality: LotQuality::Accepted,
        },
    );
    model.lots.push(Lot {
        id: "lot-filter-cartridge".into(),
        resource: "filter-cartridge".into(),
        si: 1.0,
        quality: LotQuality::Accepted,
    });
    model.add_process(Process {
        id: "replace-filter".into(),
        inputs: vec![Flow { resource: "filter-cartridge".into(), si: 1.0 }],
        outputs: vec![Flow { resource: "failed-filter".into(), si: 1.0 }],
        occupies: vec!["robot-m1".into()],
        requires_capabilities: Vec::new(),
        restores: vec!["filter-1".into()],
        duration_s: Some(3600.0),
        verification: Verification::Modeled,
        asset_mass_change_kg: 0.0,
        mass_tolerance: 0.01,
        process_yield: None,
    });
}

fn add_robot_restorer(model: &mut Model) {
    model.resources.insert(
        "robot-module".into(),
        Resource {
            id: "robot-module".into(),
            unit: &UNITS[0],
            mass_per_unit_kg: Some(1.0),
            quality: LotQuality::Accepted,
        },
    );
    model.resources.insert(
        "failed-robot".into(),
        Resource {
            id: "failed-robot".into(),
            unit: &UNITS[0],
            mass_per_unit_kg: Some(1.0),
            quality: LotQuality::Accepted,
        },
    );
    model.lots.push(Lot {
        id: "lot-robot-module".into(),
        resource: "robot-module".into(),
        si: 1.0,
        quality: LotQuality::Accepted,
    });
    model.add_process(Process {
        id: "replace-robot".into(),
        inputs: vec![Flow { resource: "robot-module".into(), si: 1.0 }],
        outputs: vec![Flow { resource: "failed-robot".into(), si: 1.0 }],
        occupies: Vec::new(),
        requires_capabilities: Vec::new(),
        restores: vec!["robot-m1".into()],
        duration_s: Some(7200.0),
        verification: Verification::Modeled,
        asset_mass_change_kg: 0.0,
        mass_tolerance: 0.01,
        process_yield: None,
    });
}

fn base_events() -> Vec<FailureEvent> {
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86_400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86_400.0, asset: "lathe-1".into() });
    events
}

fn run_case(label: &str, model: &Model, events: Vec<FailureEvent>) {
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    let legacy = simulate_multi_service_longevity_with_limits(
        model,
        &["potable-water", "workshop-operability"],
        90.0 * 86_400.0,
        &events,
        battery,
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
    let canonical = run_engine(model, &options).unwrap();
    let legacy_days = legacy.survived_s / 86_400.0;
    let canonical_days = canonical.survived_s / 86_400.0;
    let termination = match &canonical.termination {
        EngineTermination::CompletedHorizon => "completed".to_string(),
        EngineTermination::Blocked(reason) => format!("blocked:{reason}"),
        EngineTermination::IncompleteAtHorizon(reason) => format!("incomplete:{reason}"),
    };
    let classification = if (legacy_days - canonical_days).abs() < 1e-9 {
        "EXPECTED_AGREEMENT"
    } else if label == "redundancy-robot-baseline" {
        "SEMANTIC_CORRECTION_REPAIR_EXECUTOR_AT_EVENT_TIME"
    } else if legacy_days.round() == canonical_days.round() {
        "SEMANTIC_CORRECTION"
    } else {
        "UNEXPLAINED"
    };
    println!("{label}\t{legacy_days:.0}\t{canonical_days:.3}\t{termination}\t{classification}");
}

fn main() -> ExitCode {
    let base = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    println!("case\tlegacy_days\tcanonical_days\tcanonical_termination\tclassification");

    let mut filter_events = base_events();
    filter_events.push(FailureEvent { at_s: 30.0 * 86_400.0, asset: "filter-1".into() });
    filter_events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    for mode in ["filter-redundant-only", "filter-redundant+filter-restorer"] {
        let mut model = base.clone();
        redundant_pump(&mut model);
        if mode.ends_with("restorer") {
            add_filter_restorer(&mut model);
        }
        run_case(mode, &model, filter_events.clone());
    }

    let mut robot_events = base_events();
    robot_events.push(FailureEvent { at_s: 20.0 * 86_400.0, asset: "robot-m1".into() });
    robot_events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    for mode in ["robot-redundant-only", "robot-redundant+robot-restorer"] {
        let mut model = base.clone();
        redundant_pump(&mut model);
        if mode.ends_with("restorer") {
            add_robot_restorer(&mut model);
        }
        run_case(mode, &model, robot_events.clone());
    }

    let mut filter_fail = base_events();
    filter_fail.push(FailureEvent { at_s: 30.0 * 86_400.0, asset: "filter-1".into() });
    filter_fail.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    run_case("redundancy-filter-baseline", &base.clone(), filter_fail.clone());
    let mut model = base.clone();
    redundant_pump(&mut model);
    run_case("redundancy-filter-redundant-pump", &model, filter_fail);

    let mut robot_fail = base_events();
    robot_fail.push(FailureEvent { at_s: 20.0 * 86_400.0, asset: "robot-m1".into() });
    robot_fail.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    run_case("redundancy-robot-baseline", &base.clone(), robot_fail.clone());
    let mut model = base.clone();
    redundant_pump(&mut model);
    run_case("redundancy-robot-redundant-pump", &model, robot_fail);
    ExitCode::SUCCESS
}
