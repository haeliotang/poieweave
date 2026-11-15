// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase B: C7 insert regeneration (40->60 days), legacy vs canonical.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{Flow, Model, Process, Verification};
use poie_viability::engine::{run_engine, EngineOptions, EngineTermination};
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sharpen_process() -> Process {
    Process {
        id: "sharpen-insert".into(),
        inputs: vec![
            Flow { resource: "worn-insert".into(), si: 1.0 },
            Flow { resource: "electric-energy".into(), si: 0.3 * 3_600_000.0 },
        ],
        outputs: vec![Flow { resource: "cutting-insert".into(), si: 1.0 }],
        occupies: vec!["gauge-1".into()],
        requires_capabilities: Vec::new(),
        restores: Vec::new(),
        duration_s: Some(3600.0),
        verification: Verification::Modeled,
        asset_mass_change_kg: 0.0,
        mass_tolerance: 0.01,
        process_yield: None,
    }
}

fn main() -> ExitCode {
    let base = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86_400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86_400.0, asset: "lathe-1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    println!("insert_count\tmode\tlegacy_days\tcanonical_days\tdelta_days\tlegacy_energy_kwh\tcanonical_energy_kwh\tcanonical_termination\tfailure_times\tclassification");
    for count in [2.0, 3.0, 4.0] {
        for mode in ["baseline", "sharpen"] {
            let mut model = base.clone();
            if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "cutting-insert") {
                lot.si = count;
            }
            if mode == "sharpen" {
                model.add_process(sharpen_process());
            }
            let legacy = simulate_multi_service_longevity_with_limits(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86_400.0,
                &events,
                battery,
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
                load_kw: 0.0,
            };
            let canonical = run_engine(&model, &options).unwrap();
            let legacy_days = legacy.survived_s / 86_400.0;
            let canonical_days = canonical.survived_s / 86_400.0;
            let legacy_energy = legacy.final_inventory["electric-energy"] / 3_600_000.0;
            let canonical_energy = canonical.final_inventory["electric-energy"] / 3_600_000.0;
            let termination = match &canonical.termination {
                EngineTermination::CompletedHorizon => "completed".to_string(),
                EngineTermination::Blocked(reason) => format!("blocked:{reason}"),
                EngineTermination::IncompleteAtHorizon(reason) => format!("incomplete:{reason}"),
            };
            let canonical_failures: Vec<f64> = canonical
                .trace
                .iter()
                .filter(|event| event.kind.starts_with("failure:"))
                .map(|event| event.at_s)
                .collect();
            let legacy_failures: Vec<f64> =
                legacy.log.iter().filter(|entry| entry.failed.is_some()).map(|entry| entry.at_s).collect();
            let failure_times = if canonical_failures == legacy_failures { "match" } else { "differ" };
            let classification =
                if (legacy_days - canonical_days).abs() < 1e-9 && (legacy_energy - canonical_energy).abs() < 1e-6 {
                    "EXPECTED_AGREEMENT"
                } else {
                    "UNEXPLAINED"
                };
            println!(
                "{count:.0}\t{mode}\t{legacy_days:.0}\t{canonical_days:.0}\t{:.1}\t{legacy_energy:.1}\t{canonical_energy:.1}\t{termination}\t{failure_times}\t{classification}",
                canonical_days - legacy_days
            );
        }
    }
    ExitCode::SUCCESS
}
