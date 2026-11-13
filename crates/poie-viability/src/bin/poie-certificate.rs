// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Emit a machine-checkable certificate for the Base-05 site-load result.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_digest::hex_sha256;
use poie_ontology::model::Model;
use poie_viability::survival::{simulate_multi_service_longevity_with_load, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model_path = root.join("scenarios/base-05/model.toml");
    let bytes = fs::read(&model_path).expect("read model");
    let model_sha256 = hex_sha256(&bytes);
    let model = Model::load(&model_path).unwrap_or_else(|error| panic!("load Base-05: {error}"));
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86400.0, asset: "lathe-1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    let mut rows = Vec::new();
    for load_kw in [0.0, 1.0, 2.0, 3.0, 4.0] {
        let report = simulate_multi_service_longevity_with_load(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86400.0,
            &events,
            battery,
            load_kw,
        )
        .unwrap();
        let blocked = report.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
        rows.push(serde_json::json!({
            "load_kw": load_kw,
            "survived_days": report.survived_s / 86400.0,
            "blocked_by": blocked,
        }));
    }
    let certificate = serde_json::json!({
        "schema": "poie.viability-certificate/1",
        "experiment": "s2-load",
        "model": "scenarios/base-05/model.toml",
        "model_sha256": model_sha256,
        "horizon_days": 90,
        "battery": {"capacity_kwh": 200.0, "charge_kw": 10.0},
        "rows": rows,
        "conclusion": "0-2 kW remains material-limited at 40 days; 3-4 kW becomes energy-limited",
        "claim_scope": "model/simulation only; not a physical safety or lifetime claim",
    });
    println!("{}", serde_json::to_string_pretty(&certificate).expect("certificate JSON"));
    ExitCode::SUCCESS
}
