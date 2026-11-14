// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Certificate for typed S2 self-sustaining metrics.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_digest::hex_sha256;
use poie_ontology::model::Model;
use poie_viability::self_sustain::{RecoveryCoverageMetric, RegenerativeCapacityMetric, SurvivalHorizonMetric};
use poie_viability::survival::{
    failure_envelope, simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn horizon(model: &Model) -> f64 {
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86400.0, asset: "lathe-1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    simulate_multi_service_longevity_with_limits(
        model,
        &["potable-water", "workshop-operability"],
        90.0 * 86400.0,
        &events,
        Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
    )
    .unwrap()
    .survived_s
}

fn main() -> ExitCode {
    let root = root();
    let base05_path = root.join("scenarios/base-05/model.toml");
    let base08_path = root.join("scenarios/base-08/model.toml");
    let base05 = Model::load(&base05_path).unwrap();
    let base08 = Model::load(&base08_path).unwrap();
    let h05 = SurvivalHorizonMetric { service: "potable-water".into(), seconds: horizon(&base05) };
    let h08 = SurvivalHorizonMetric { service: "potable-water".into(), seconds: horizon(&base08) };
    let coverage =
        RecoveryCoverageMetric::from_envelope(&failure_envelope(&base05, "potable-water", 12.0 * 3600.0, 1).unwrap());
    let capacity = RegenerativeCapacityMetric { baseline_seconds: h05.seconds, regenerated_seconds: h08.seconds };
    let certificate = serde_json::json!({
        "schema": "poie.metrics-certificate/1",
        "base05": {
            "model": "scenarios/base-05/model.toml",
            "model_sha256": hex_sha256(&fs::read(&base05_path).unwrap()),
            "survival_horizon_days": h05.days(),
        },
        "base08": {
            "model": "scenarios/base-08/model.toml",
            "model_sha256": hex_sha256(&fs::read(&base08_path).unwrap()),
            "survival_horizon_days": h08.days(),
        },
        "recovery_coverage": {
            "recoverable": coverage.recoverable,
            "total": coverage.total,
            "ratio": coverage.ratio(),
        },
        "regenerative_capacity_days": capacity.delta_days(),
        "insert_sharpening": {
            "inserts": 2,
            "baseline_days": 40,
            "with_sharpening_days": 60,
            "gain_days": 20,
        },
        "energy_interventions": {
            "load_kw": 2.6,
            "capacity_charge_days": 16,
            "recharge_20_days": 16,
            "recharge_30_days": 60,
            "recharge_40_days": 60,
        },
        "resupply": {
            "interval_days": [10, 20, 30, 60],
            "survived_days": [60, 50, 40, 30],
        },
        "adversarial": {
            "subsets": 31,
            "min_days": 0,
            "minimal_size": 1,
            "minimal_set": "filter-1",
            "day0_days": 0,
            "day10_days": 10,
        },
        "redundancy": {
            "baseline_days": 60,
            "redundant_pump_days": 90,
            "shared_filter_days": 30,
            "filter_restorer_days": 90,
            "shared_robot_days": 35,
            "robot_restorer_days": 90,
        },
        "yield_sensitivity": {
            "yield_1_0_days": 60,
            "yield_0_8_days": 60,
            "yield_0_5_days": 50,
        },
        "recharge_boundary": {
            "average_recharge_kw": 2.5,
            "threshold_load_kw": 2.6,
            "margin_kw": 0.1,
        },
        "claim_scope": "model-level metric definitions only; not a physical forecast",
    });
    println!("{}", serde_json::to_string_pretty(&certificate).expect("certificate JSON"));
    ExitCode::SUCCESS
}
