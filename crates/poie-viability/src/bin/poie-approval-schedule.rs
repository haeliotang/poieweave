// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S3: time-varying approval decisions for later recovery proposals.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{
    simulate_recovery_with_approval_schedule, simulate_recovery_with_policy, ApprovalDecision, FailureEvent,
    RecoveryPolicy,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let events = vec![
        FailureEvent { at_s: 10.0 * 86_400.0, asset: "pump-p1".into() },
        FailureEvent { at_s: 20.0 * 86_400.0, asset: "pump-p1".into() },
    ];
    let policy =
        RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 8.0 * 3600.0, approved_plan_digest: None };
    println!("case\treceipts\tsurvived_days\tblocked_by");
    let all = simulate_recovery_with_policy(&model, "potable-water", 30.0 * 86_400.0, &events, policy.clone()).unwrap();
    let all_receipts = all.log.iter().filter(|entry| entry.receipt.is_some()).count();
    let all_blocked = all.log.iter().find_map(|entry| entry.blocked_by.clone()).unwrap_or_else(|| "none".into());
    println!("approved-all\t{all_receipts}\t{:.0}\t{all_blocked}", all.survived_s / 86_400.0);

    let schedule = vec![(0.0, ApprovalDecision::Approved), (20.0 * 86_400.0, ApprovalDecision::Rejected)];
    let later =
        simulate_recovery_with_approval_schedule(&model, "potable-water", 30.0 * 86_400.0, &events, policy, &schedule)
            .unwrap();
    let later_receipts = later.log.iter().filter(|entry| entry.receipt.is_some()).count();
    let later_blocked = later.log.iter().find_map(|entry| entry.blocked_by.clone()).unwrap_or_else(|| "none".into());
    println!("reject-day20\t{later_receipts}\t{:.0}\t{later_blocked}", later.survived_s / 86_400.0);
    ExitCode::SUCCESS
}
