// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S5-V native canonical observation for the external energy challenge.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine_observed, EngineOptions, ObservationPhase, ObservationRequest};
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
    let mut requests: Vec<ObservationRequest> = [0.0, 3600.0, 7200.0, 10800.0, 284400.0]
        .into_iter()
        .map(|at_s| ObservationRequest { at_s, phase: ObservationPhase::After })
        .collect();
    requests.push(ObservationRequest { at_s: 10800.0, phase: ObservationPhase::Before });
    let observed = run_engine_observed(&model, &options, &requests).unwrap();
    let termination = match &observed.outcome.termination {
        poie_viability::engine::EngineTermination::CompletedHorizon => "completed".to_string(),
        poie_viability::engine::EngineTermination::Blocked(reason) => format!("blocked:{reason}"),
        poie_viability::engine::EngineTermination::IncompleteAtHorizon(reason) => format!("incomplete:{reason}"),
    };
    println!("termination\tsurvived_s\tsamples");
    println!("{termination}\t{:.3}\t{}", observed.outcome.survived_s, observed.observations.len());
    println!("at_s\tstatus\tenergy_kwh\tgenerated_kwh\tload_kwh\trepair_kwh\tcurtailed_kwh\tresupply_kwh\trepair_active\trepair_label\tphase");
    for row in observed.observations {
        println!(
            "{:.0}\t{}\t{:.6}\t{:.6}\t{:.6}\t{:.6}\t{:.6}\t{:.6}\t{}\t{}\t{:?}",
            row.at_s,
            row.status.label(),
            row.inventory.get("electric-energy").copied().unwrap_or(0.0) / 3_600_000.0,
            row.energy_generated_j / 3_600_000.0,
            row.energy_load_j / 3_600_000.0,
            row.energy_repair_j / 3_600_000.0,
            row.energy_curtailed_j / 3_600_000.0,
            row.energy_resupply_j / 3_600_000.0,
            row.running_repair.is_some(),
            row.running_repair.as_ref().map(|r| r.label.as_str()).unwrap_or("NONE"),
            row.phase,
        );
    }
    ExitCode::SUCCESS
}
