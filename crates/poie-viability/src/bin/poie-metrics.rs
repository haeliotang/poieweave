// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 self-sustaining metrics on Base-05/Base-08.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

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
    let base05 = Model::load(&root.join("scenarios/base-05/model.toml")).unwrap();
    let base08 = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let h05 = horizon(&base05);
    let h08 = horizon(&base08);
    let envelope = failure_envelope(&base05, "potable-water", 12.0 * 3600.0, 1).unwrap();
    let coverage = RecoveryCoverageMetric::from_envelope(&envelope);
    let horizon05 = SurvivalHorizonMetric { service: "potable-water".into(), seconds: h05 };
    let horizon08 = SurvivalHorizonMetric { service: "potable-water".into(), seconds: h08 };
    let capacity = RegenerativeCapacityMetric { baseline_seconds: h05, regenerated_seconds: h08 };
    println!("survival_horizon_base05_days={:.1}", horizon05.days());
    println!("survival_horizon_base08_days={:.1}", horizon08.days());
    println!("recovery_coverage={}/{} ratio={:.6}", coverage.recoverable, coverage.total, coverage.ratio());
    println!("regenerative_capacity_days={:.1}", capacity.delta_days());
    ExitCode::SUCCESS
}
