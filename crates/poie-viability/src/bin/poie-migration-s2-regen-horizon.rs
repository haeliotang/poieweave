// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase B: certificate-s2-regeneration Base-05/08 interval rows, legacy vs canonical.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions, EngineTermination};
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("model\tinterval_days\tlegacy_days\tcanonical_days\tlegacy_energy_kwh\tcanonical_energy_kwh\tcanonical_termination\tclassification");
    for model_name in ["base-05", "base-08"] {
        let model = Model::load(&root().join(format!("scenarios/{model_name}/model.toml"))).unwrap();
        for interval in [10_u32, 15, 20, 30] {
            let mut events: Vec<FailureEvent> = (1..=(89 / interval))
                .map(|index| FailureEvent { at_s: f64::from(index * interval) * 86_400.0, asset: "pump-p1".into() })
                .collect();
            events.push(FailureEvent { at_s: 35.0 * 86_400.0, asset: "lathe-1".into() });
            events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
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
            let classification =
                if (legacy_days - canonical_days).abs() < 1e-9 && (legacy_energy - canonical_energy).abs() < 1e-6 {
                    "EXPECTED_AGREEMENT"
                } else {
                    "UNEXPLAINED"
                };
            println!("{model_name}\t{interval}\t{legacy_days:.3}\t{canonical_days:.3}\t{legacy_energy:.1}\t{canonical_energy:.1}\t{termination}\t{classification}");
        }
    }
    ExitCode::SUCCESS
}
