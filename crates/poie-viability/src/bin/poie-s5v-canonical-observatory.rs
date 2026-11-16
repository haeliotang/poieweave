// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S5-V canonical trajectory observatory for the external energy challenge.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::EngineOptions;
use poie_viability::observer::observe_states;
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
    let times = [0.0, 3600.0, 7200.0, 10800.0, 79.0 * 3600.0];
    let observed = observe_states(&model, &options, &times).unwrap();
    println!("at_s\tenergy_kwh\tgenerated_kwh\tload_kwh\trepair_kwh\tcurtailed_kwh\trepair_active\trepair_detail\trepair_state_source\ttermination");
    for row in observed {
        println!(
            "{:.0}\t{:.6}\t{:.6}\t{:.6}\t{:.6}\t{:.6}\t{}\t{}\t{}\t{}",
            row.at_s,
            row.energy_j / 3_600_000.0,
            row.energy_generated_j / 3_600_000.0,
            row.energy_load_j / 3_600_000.0,
            row.energy_repair_j / 3_600_000.0,
            row.energy_curtailed_j / 3_600_000.0,
            row.repair_active,
            row.repair_detail.as_deref().unwrap_or("NOT_EXPORTED"),
            row.repair_state_source,
            row.termination,
        );
    }
    ExitCode::SUCCESS
}
