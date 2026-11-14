// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S3 lease boundary: plan duration equal to the lease.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_recovery_with_policy, ApprovalDecision, RecoveryPolicy};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    println!("lease_s\texecuted\tblocked_by\treceipts");
    for lease_s in [10_799.0, 10_800.0, 10_801.0] {
        let policy = RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s, approved_plan_digest: None };
        let report = simulate_recovery_with_policy(&model, "potable-water", 86_400.0, &[], policy).unwrap();
        let blocked = report.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
        let executed = report.log.iter().any(|entry| !entry.repaired.is_empty());
        let receipts = report.log.iter().filter(|entry| entry.receipt.is_some()).count();
        println!("{lease_s:.0}\t{executed}\t{blocked}\t{receipts}");
    }
    ExitCode::SUCCESS
}
