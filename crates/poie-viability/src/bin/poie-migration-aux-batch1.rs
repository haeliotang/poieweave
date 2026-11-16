// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase E sample: energy-boundary and held-out auxiliary experiments.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions, EngineTermination};
use poie_viability::survival::{
    simulate_multi_service_longevity_with_limits, simulate_multi_service_longevity_with_load, BatteryLimits,
    FailureEvent,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn base_events() -> Vec<FailureEvent> {
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86_400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86_400.0, asset: "lathe-1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    events
}

fn canonical(
    model: &Model,
    battery: BatteryLimits,
    load_kw: f64,
    events: Vec<FailureEvent>,
    horizon_s: f64,
) -> (f64, String) {
    let options = EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s,
        failures: events,
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(battery),
        load_kw,
    };
    let outcome = run_engine(model, &options).unwrap();
    let termination = match &outcome.termination {
        EngineTermination::CompletedHorizon => "completed".to_string(),
        EngineTermination::Blocked(reason) => format!("blocked:{reason}"),
        EngineTermination::IncompleteAtHorizon(reason) => format!("incomplete:{reason}"),
    };
    (outcome.survived_s / 86_400.0, termination)
}

fn main() -> ExitCode {
    let base08 = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    println!("experiment\trow\tlegacy_days\tcanonical_days\tclassification");
    for load_kw in [2.0_f64, 3.0] {
        for capacity in [100.0_f64, 200.0, 300.0] {
            for charge in [3.0_f64, 10.0, 30.0] {
                let battery = BatteryLimits { capacity_kwh: capacity, charge_kw: charge };
                let legacy = simulate_multi_service_longevity_with_load(
                    &base08,
                    &["potable-water", "workshop-operability"],
                    90.0 * 86_400.0,
                    &base_events(),
                    Some(battery),
                    load_kw,
                )
                .unwrap();
                let (canonical_days, _) = canonical(&base08, battery, load_kw, base_events(), 90.0 * 86_400.0);
                let legacy_days = legacy.survived_s / 86_400.0;
                let classification =
                    if legacy_days.round() == canonical_days.round() { "EXPECTED_AGREEMENT" } else { "UNEXPLAINED" };
                println!("energy-boundary\tload={load_kw:.0},cap={capacity:.0},charge={charge:.0}\t{legacy_days:.0}\t{canonical_days:.3}\t{classification}");
            }
        }
    }
    let base05 = Model::load(&root().join("scenarios/base-05/model.toml")).unwrap();
    for interval in [12_u32, 25] {
        let mut events: Vec<FailureEvent> = (1..=(89 / interval))
            .map(|index| FailureEvent { at_s: f64::from(index * interval) * 86_400.0, asset: "pump-p1".into() })
            .collect();
        events.push(FailureEvent { at_s: 35.0 * 86_400.0, asset: "lathe-1".into() });
        events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
        let legacy = simulate_multi_service_longevity_with_limits(
            &base05,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &events,
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        )
        .unwrap();
        let (canonical_days, _) =
            canonical(&base05, BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }, 0.0, events, 90.0 * 86_400.0);
        let legacy_days = legacy.survived_s / 86_400.0;
        let classification = if (legacy_days - canonical_days).abs() < 1e-9 {
            "EXPECTED_AGREEMENT"
        } else if legacy_days.round() == canonical_days.round() {
            "SEMANTIC_CORRECTION"
        } else {
            "UNEXPLAINED"
        };
        println!("heldout\tinterval={interval}\t{legacy_days:.0}\t{canonical_days:.3}\t{classification}");
    }
    ExitCode::SUCCESS
}
