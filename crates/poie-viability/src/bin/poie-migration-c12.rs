// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase B: C12 grease boundary under canonical repair-time semantics.

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

fn main() -> ExitCode {
    let base = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let events: Vec<FailureEvent> =
        (1..=15).map(|index| FailureEvent { at_s: f64::from(index * 2) * 86_400.0, asset: "pump-p1".into() }).collect();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("grease_kg\tinstant_days\tevent_days\tcanonical_days\tevent_energy_kwh\tcanonical_energy_kwh\tcanonical_termination\tclassification");
    for grease in [0.10, 0.15, 0.20, 0.30, 0.50] {
        let mut model = base.clone();
        for lot in &mut model.lots {
            if lot.resource == "sleeve-p1" {
                lot.si = 0.0;
            }
            if lot.resource == "grease" {
                lot.si = grease;
            }
        }
        model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap().duration_s =
            Some(5.0 * 86_400.0);
        let instant = simulate_multi_service_longevity_with_load(
            &model,
            &["potable-water", "workshop-operability"],
            30.0 * 86_400.0,
            &events,
            battery,
            0.0,
        )
        .unwrap();
        let event = simulate_multi_service_longevity_event_driven(
            &model,
            &["potable-water", "workshop-operability"],
            30.0 * 86_400.0,
            &events,
            battery,
            0.0,
        )
        .unwrap();
        let options = EngineOptions {
            services: vec!["potable-water", "workshop-operability"],
            horizon_s: 30.0 * 86_400.0,
            failures: events.clone(),
            resupplies: Vec::new(),
            resources: vec!["robot-m1".into(), "lathe-1".into()],
            alternative_pool: vec!["robot-m1".into()],
            battery,
            load_kw: 0.0,
        };
        let canonical = run_engine(&model, &options).unwrap();
        let event_days = event.survived_s / 86_400.0;
        let canonical_days = canonical.survived_s / 86_400.0;
        let event_energy = event.final_inventory["electric-energy"] / 3_600_000.0;
        let canonical_energy = canonical.final_inventory["electric-energy"] / 3_600_000.0;
        let termination = match &canonical.termination {
            EngineTermination::CompletedHorizon => "completed".to_string(),
            EngineTermination::Blocked(reason) => format!("blocked:{reason}"),
            EngineTermination::IncompleteAtHorizon(reason) => format!("incomplete:{reason}"),
        };
        let classification = if (event_days - canonical_days).abs() < 1e-9
            && (event_energy - canonical_energy).abs() < 1e-6
        {
            "EXPECTED_AGREEMENT"
        } else if matches!(&canonical.termination, EngineTermination::Blocked(reason) if reason.contains("interrupted"))
        {
            "SEMANTIC_CORRECTION_IN_REPAIR_FAILURE"
        } else {
            "UNEXPLAINED"
        };
        println!(
            "{grease:.2}\t{:.0}\t{event_days:.0}\t{canonical_days:.0}\t{event_energy:.1}\t{canonical_energy:.1}\t{termination}\t{classification}",
            instant.survived_s / 86_400.0
        );
    }
    ExitCode::SUCCESS
}
