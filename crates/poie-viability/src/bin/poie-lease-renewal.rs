// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S3: is the lease per proposal (implicitly renewed) or absolute?

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_recovery_with_policy, ApprovalDecision, FailureEvent, RecoveryPolicy};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let events: Vec<FailureEvent> = [10.0, 20.0, 30.0]
        .into_iter()
        .map(|days| FailureEvent { at_s: days * 86_400.0, asset: "pump-p1".into() })
        .collect();
    let policy =
        RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 8.0 * 3600.0, approved_plan_digest: None };
    let report = simulate_recovery_with_policy(&model, "potable-water", 90.0 * 86_400.0, &events, policy).unwrap();
    let receipts = report.log.iter().filter(|entry| entry.receipt.is_some()).count();
    let blocked = report.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
    println!(
        "events=3 lease_h=8 receipts={receipts} survived_days={:.0} blocked_by={blocked}",
        report.survived_s / 86_400.0
    );
    ExitCode::SUCCESS
}
