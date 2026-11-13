// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Export S3 recovery receipts as JSON for an independent verifier.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_recovery_with_policy, ApprovalDecision, FailureEvent, RecoveryPolicy};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-05/model.toml")).unwrap_or_else(|error| {
        panic!("load Base-05: {error}");
    });
    let policy =
        RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 8.0 * 3600.0, approved_plan_digest: None };
    let events = vec![FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p1".into() }];
    let report = simulate_recovery_with_policy(&model, "potable-water", 30.0 * 86400.0, &events, policy).unwrap();
    let receipts: Vec<_> = report.log.iter().filter_map(|entry| entry.receipt.clone()).collect();
    println!("{}", serde_json::to_string_pretty(&receipts).expect("receipt JSON"));
    ExitCode::SUCCESS
}
