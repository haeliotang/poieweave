// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S3 execution receipt bound to the recovered plan and inventory state.

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
    for entry in &report.log {
        if let Some(receipt) = &entry.receipt {
            println!(
                "at_day={:.0}\tplan_digest={}\treceipt_digest={}\tverified={}",
                receipt.at_s / 86400.0,
                receipt.plan_digests.first().map(String::as_str).unwrap_or("none"),
                receipt.digest,
                receipt.verify()
            );
        }
    }
    ExitCode::SUCCESS
}
