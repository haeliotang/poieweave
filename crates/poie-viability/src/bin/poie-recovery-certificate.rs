// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Emit a certificate for an approved S3 recovery trace.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_digest::hex_sha256;
use poie_ontology::model::Model;
use poie_viability::survival::{simulate_recovery_with_policy, ApprovalDecision, FailureEvent, RecoveryPolicy};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model_path = root.join("scenarios/base-05/model.toml");
    let model_sha256 = hex_sha256(&fs::read(&model_path).expect("read model"));
    let model = Model::load(&model_path).unwrap_or_else(|error| panic!("load Base-05: {error}"));
    let policy =
        RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 8.0 * 3600.0, approved_plan_digest: None };
    let events = vec![FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p1".into() }];
    let report = simulate_recovery_with_policy(&model, "potable-water", 30.0 * 86400.0, &events, policy).unwrap();
    let receipts: Vec<_> = report.log.iter().filter_map(|entry| entry.receipt.clone()).collect();
    let certificate = serde_json::json!({
        "schema": "poie.recovery-certificate/1",
        "model": "scenarios/base-05/model.toml",
        "model_sha256": model_sha256,
        "service": "potable-water",
        "horizon_days": 30,
        "approval": {"decision": "approved", "lease_s": 28800.0},
        "events": [{"at_s": 864000.0, "asset": "pump-p1"}],
        "receipts": receipts,
        "claim_scope": "isolated simulation only; not a hardware authority or safety claim",
    });
    println!("{}", serde_json::to_string_pretty(&certificate).expect("certificate JSON"));
    ExitCode::SUCCESS
}
