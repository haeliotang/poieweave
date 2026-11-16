// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S5-V reference export: canonical energy-only run with no failures.
//!
//! This is not a new simulation model; it exports the existing canonical
//! engine ledger for the external FMU challenge. The scenario is Base-08 with
//! no failure events, load 3.0 kW, battery 200 kWh / 10 kW charge limit.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions, EngineTermination};
use poie_viability::survival::BatteryLimits;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let options = EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s: 200.0 * 3600.0,
        failures: Vec::new(),
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        load_kw: 3.0,
    };
    let outcome = run_engine(&model, &options).unwrap();
    let terminal = match &outcome.termination {
        EngineTermination::CompletedHorizon => "completed".to_string(),
        EngineTermination::Blocked(reason) => format!("blocked:{reason}"),
        EngineTermination::IncompleteAtHorizon(reason) => format!("incomplete:{reason}"),
    };
    println!("termination\tsurvived_h\tsurvived_s");
    println!("{terminal}\t{:.6}\t{:.3}", outcome.survived_s / 3600.0, outcome.survived_s);
    println!("at_s\tenergy_generated_kwh\tenergy_load_kwh\tenergy_repair_kwh\tenergy_curtailed_kwh\tenergy_before_j\tenergy_after_j");
    for entry in &outcome.trace {
        println!(
            "{:.3}\t{:.6}\t{:.6}\t{:.6}\t{:.6}\t{:.3}\t{:.3}",
            entry.at_s,
            entry.energy_generated_j / 3_600_000.0,
            entry.energy_load_j / 3_600_000.0,
            entry.energy_repair_j / 3_600_000.0,
            entry.energy_curtailed_j / 3_600_000.0,
            entry.inventory_before.get("electric-energy").copied().unwrap_or(0.0),
            entry.inventory_after.get("electric-energy").copied().unwrap_or(0.0),
        );
    }
    ExitCode::SUCCESS
}
