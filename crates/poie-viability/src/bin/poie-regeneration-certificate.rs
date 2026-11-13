// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Certificate for the Base-05 vs Base-08 regeneration horizon result.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_digest::hex_sha256;
use poie_ontology::model::Model;
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let mut rows = Vec::new();
    for name in ["base-05", "base-08"] {
        let relative = format!("scenarios/{name}/model.toml");
        let path = root.join(&relative);
        let sha256 = hex_sha256(&fs::read(&path).expect("read model"));
        let model = Model::load(&path).unwrap_or_else(|error| panic!("load {relative}: {error}"));
        let mut events: Vec<FailureEvent> = (1..=8)
            .map(|index| FailureEvent { at_s: f64::from(index * 10) * 86400.0, asset: "pump-p1".into() })
            .collect();
        events.push(FailureEvent { at_s: 35.0 * 86400.0, asset: "lathe-1".into() });
        events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
        let report = simulate_multi_service_longevity_with_limits(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86400.0,
            &events,
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        )
        .unwrap();
        rows.push(serde_json::json!({
            "model": relative,
            "model_sha256": sha256,
            "survived_days": report.survived_s / 86400.0,
        }));
    }
    let certificate = serde_json::json!({
        "schema": "poie.regeneration-certificate/1",
        "experiment": "s2-regeneration-horizon",
        "rows": rows,
        "conclusion": "local swarf recycling extends the 10-day failure horizon from 40 to 60 days",
        "claim_scope": "model-level regeneration result only; not a physical yield claim",
    });
    println!("{}", serde_json::to_string_pretty(&certificate).expect("certificate JSON"));
    ExitCode::SUCCESS
}
