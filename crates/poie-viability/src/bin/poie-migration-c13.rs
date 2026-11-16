// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase B: C13 energy-boundary invariance under canonical repair time.

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
    let mut model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap().duration_s =
        Some(5.0 * 86_400.0);
    let events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86_400.0, asset: "pump-p1".into() }).collect();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("load_kw\tinstant_days\tevent_days\tcanonical_days\tevent_energy_kwh\tcanonical_energy_kwh\tcanonical_termination\tclassification");
    for load_kw in [2.6, 3.0, 4.0] {
        let instant = simulate_multi_service_longevity_with_load(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &events,
            battery,
            load_kw,
        )
        .unwrap();
        let event = simulate_multi_service_longevity_event_driven(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &events,
            battery,
            load_kw,
        )
        .unwrap();
        let options = EngineOptions {
            services: vec!["potable-water", "workshop-operability"],
            horizon_s: 90.0 * 86_400.0,
            failures: events.clone(),
            resupplies: Vec::new(),
            resources: vec!["robot-m1".into(), "lathe-1".into()],
            alternative_pool: vec!["robot-m1".into()],
            battery,
            load_kw,
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
        let classification =
            if (event_days - canonical_days).abs() < 1e-9 && (event_energy - canonical_energy).abs() < 1e-6 {
                "EXPECTED_AGREEMENT"
            } else if event_days.round() == canonical_days.round() {
                "SEMANTIC_CORRECTION_REPAIR_SCHEDULE"
            } else {
                "UNEXPLAINED"
            };
        println!(
            "{load_kw:.1}\t{:.0}\t{event_days:.3}\t{canonical_days:.3}\t{event_energy:.1}\t{canonical_energy:.1}\t{termination}\t{classification}",
            instant.survived_s / 86_400.0
        );
    }
    ExitCode::SUCCESS
}
