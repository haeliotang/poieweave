// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 joint-conflict check: independent coverage versus one shared inventory.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{
    failure_envelope, simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-06/model.toml")).unwrap_or_else(|error| {
        panic!("load Base-06: {error}");
    });
    let pair = vec!["pump-p1".to_string(), "pump-p2".to_string()];
    let envelope = failure_envelope(&model, "potable-water", 12.0 * 3600.0, 2).unwrap();
    let independent =
        envelope.cases.iter().find(|case| case.failed == pair).map(|case| case.recoverable).unwrap_or(false);
    let events = vec![
        FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p1".into() },
        FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p2".into() },
    ];
    let joint = simulate_multi_service_longevity_with_limits(
        &model,
        &["potable-water"],
        30.0 * 86400.0,
        &events,
        Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
    )
    .unwrap();
    let blocked = joint.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
    println!("pair=failed={},{}", pair[0], pair[1]);
    println!("independent_coverage={independent}");
    println!("joint_survived_days={:.0}", joint.survived_s / 86400.0);
    println!("joint_blocked_by={blocked}");
    ExitCode::SUCCESS
}
