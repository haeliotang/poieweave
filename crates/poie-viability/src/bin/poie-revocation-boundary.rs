// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S3 revocation timing relative to the 3 h plan.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_recovery_with_revocation, ApprovalDecision, RecoveryPolicy};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    println!("revoke_at_s\texecuted\tsurvived_s\tblocked_by\treceipts");
    for revoke_at_s in [0.0, 1.0, 10_799.0, 10_800.0] {
        let policy =
            RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 8.0 * 3600.0, approved_plan_digest: None };
        let report =
            simulate_recovery_with_revocation(&model, "potable-water", 86_400.0, &[], policy, revoke_at_s).unwrap();
        let blocked = report.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
        let executed = report.log.iter().any(|entry| !entry.repaired.is_empty());
        let receipts = report.log.iter().filter(|entry| entry.receipt.is_some()).count();
        println!("{revoke_at_s:.0}\t{executed}\t{:.0}\t{blocked}\t{receipts}", report.survived_s);
    }
    ExitCode::SUCCESS
}
