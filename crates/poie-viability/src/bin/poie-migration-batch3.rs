// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S5 migration batch 3: resupply, load and battery boundaries.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{AssetStatus, Lot, LotQuality, Model};
use poie_viability::engine::{run_engine, EngineOptions, EngineTermination};
use poie_viability::survival::{
    simulate_multi_service_longevity_event_driven_with_resupply, simulate_multi_service_longevity_with_load,
    BatteryLimits, Resupply,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn base08() -> Model {
    Model::load(&root().join("scenarios/base-08/model.toml")).unwrap()
}

fn set_energy(model: &mut Model, kwh: f64) {
    model.lots.retain(|lot| lot.resource != "electric-energy");
    model.lots.push(Lot {
        id: "test-energy".into(),
        resource: "electric-energy".into(),
        si: kwh * 3_600_000.0,
        quality: LotQuality::Accepted,
    });
}

fn main() -> ExitCode {
    println!("scenario\tcanonical_horizon_h\tlegacy_horizon_h\tcanonical_energy_kwh\tclassification");
    // 3B: load over a repair window.
    let mut model = base08();
    set_energy(&mut model, 10.0);
    model.assets.get_mut("pv-1").unwrap().status = AssetStatus::Failed;
    model.assets.get_mut("pump-p1").unwrap().status = AssetStatus::Failed;
    for lot in &mut model.lots {
        if lot.resource == "sleeve-p1" {
            lot.si = 1.0;
        }
    }
    let options = EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s: 5.0 * 3600.0,
        failures: Vec::new(),
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: None,
        load_kw: 1.0,
    };
    let canonical = run_engine(&model, &options).unwrap();
    let legacy = simulate_multi_service_longevity_with_load(
        &model,
        &["potable-water", "workshop-operability"],
        5.0 * 3600.0,
        &[],
        None,
        1.0,
    )
    .unwrap();
    println!(
        "load-repair\t{:.1}\t{:.1}\t{:.1}\tload-integrates",
        canonical.survived_s / 3600.0,
        legacy.survived_s / 3600.0,
        canonical.final_inventory["electric-energy"] / 3_600_000.0
    );

    // 3A: resupply before arrival is not usable.
    let mut model = base08();
    model.assets.get_mut("pump-p1").unwrap().status = AssetStatus::Failed;
    for lot in &mut model.lots {
        if lot.resource == "sleeve-p1" {
            lot.si = 1.0;
        }
        if lot.resource == "grease" {
            lot.si = 0.0;
        }
    }
    let supply = Resupply { at_s: 3600.0, resource: "grease".into(), si: 1.0 };
    let options = EngineOptions {
        services: vec!["potable-water"],
        horizon_s: 4.0 * 3600.0,
        failures: Vec::new(),
        resupplies: vec![supply.clone()],
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: None,
        load_kw: 0.0,
    };
    let canonical = run_engine(&model, &options).unwrap();
    let legacy = simulate_multi_service_longevity_event_driven_with_resupply(
        &model,
        &["potable-water"],
        4.0 * 3600.0,
        &[],
        &[supply],
    )
    .unwrap();
    let canonical_label = match &canonical.termination {
        EngineTermination::CompletedHorizon => "completed".to_string(),
        EngineTermination::Blocked(reason) => format!("blocked:{reason}"),
        EngineTermination::IncompleteAtHorizon(reason) => format!("incomplete:{reason}"),
    };
    println!(
        "resupply-before-arrival\t{:.1}\t{:.1}\t{:.1}\t{}",
        canonical.survived_s / 3600.0,
        legacy.survived_s / 3600.0,
        canonical.final_inventory["electric-energy"] / 3_600_000.0,
        canonical_label
    );

    // 3C: initial energy over battery capacity.
    let mut model = base08();
    set_energy(&mut model, 10.0);
    let options = EngineOptions {
        services: vec!["potable-water"],
        horizon_s: 3600.0,
        failures: Vec::new(),
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(BatteryLimits { capacity_kwh: 1.0, charge_kw: 10.0 }),
        load_kw: 0.0,
    };
    let canonical = run_engine(&model, &options).is_err();
    let legacy = simulate_multi_service_longevity_with_load(
        &model,
        &["potable-water"],
        3600.0,
        &[],
        Some(BatteryLimits { capacity_kwh: 1.0, charge_kw: 10.0 }),
        0.0,
    )
    .is_err();
    println!("initial-overcapacity\t0.0\t0.0\t10.0\tcanonical_reject={canonical},legacy_reject={legacy}");

    ExitCode::SUCCESS
}
