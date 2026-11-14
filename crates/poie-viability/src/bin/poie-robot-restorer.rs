// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 does a manual robot restorer remove the shared repair-executor blocker?

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{Asset, AssetStatus, Flow, Lot, LotQuality, Model, Process, Resource, Verification};
use poie_ontology::units::UNITS;
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

fn main() -> ExitCode {
    let root = root();
    let base = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86400.0, asset: "lathe-1".into() });
    events.push(FailureEvent { at_s: 20.0 * 86400.0, asset: "robot-m1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    println!("mode\tsurvived_days\tblocked_by");
    for mode in ["redundant-only", "redundant+robot-restorer"] {
        let mut model = base.clone();
        redundant_pump(&mut model);
        if mode.ends_with("restorer") {
            add_robot_restorer(&mut model);
        }
        let report = simulate_multi_service_longevity_with_limits(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86400.0,
            &events,
            battery,
        )
        .unwrap();
        let blocked = report.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
        println!("{mode}\t{:.0}\t{blocked}", report.survived_s / 86400.0);
    }
    ExitCode::SUCCESS
}
