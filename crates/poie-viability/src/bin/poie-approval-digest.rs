// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S3 approval bound to a plan digest.

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
    let empty: Vec<FailureEvent> = Vec::new();
    let report = simulate_recovery_with_policy(&model, "potable-water", 30.0 * 86400.0, &empty, policy).unwrap();
    let correct = report.log[0].plan_digests[0].clone();
    let wrong = "0".repeat(64);
    println!("correct_plan_digest={correct}");
    for (label, digest) in [("correct", correct.clone()), ("wrong", wrong)] {
        let policy = RecoveryPolicy {
            decision: ApprovalDecision::Approved,
            lease_s: 8.0 * 3600.0,
            approved_plan_digest: Some(digest),
        };
        let run = simulate_recovery_with_policy(&model, "potable-water", 30.0 * 86400.0, &empty, policy).unwrap();
        let blocked = run.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
        println!("{label}\tsurvived_days={:.0}\tblocked_by={blocked}", run.survived_s / 86400.0);
    }
    let stale = RecoveryPolicy {
        decision: ApprovalDecision::Approved,
        lease_s: 8.0 * 3600.0,
        approved_plan_digest: Some(correct),
    };
    let changed = simulate_recovery_with_policy(
        &model,
        "potable-water",
        30.0 * 86400.0,
        &[FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p1".into() }],
        stale,
    )
    .unwrap();
    let blocked = changed.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
    println!("changed_plan\tsurvived_days={:.0}\tblocked_by={blocked}", changed.survived_s / 86400.0);
    ExitCode::SUCCESS
}
