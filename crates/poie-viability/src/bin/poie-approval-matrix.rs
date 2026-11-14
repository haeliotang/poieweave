// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S3 approval/lease/revocation matrix on the same failure.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{
    simulate_recovery_with_policy, simulate_recovery_with_revocation, ApprovalDecision, FailureEvent, RecoveryPolicy,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let events = vec![FailureEvent { at_s: 10.0 * 86_400.0, asset: "pump-p1".into() }];
    println!("case\texecuted\tsurvived_days\tblocked_by");
    let cases: Vec<(&str, RecoveryPolicy, Option<f64>)> = vec![
        (
            "required",
            RecoveryPolicy { decision: ApprovalDecision::Required, lease_s: 8.0 * 3600.0, approved_plan_digest: None },
            None,
        ),
        (
            "rejected",
            RecoveryPolicy { decision: ApprovalDecision::Rejected, lease_s: 8.0 * 3600.0, approved_plan_digest: None },
            None,
        ),
        (
            "lease-1h",
            RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 3600.0, approved_plan_digest: None },
            None,
        ),
        (
            "approved-8h",
            RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 8.0 * 3600.0, approved_plan_digest: None },
            None,
        ),
        (
            "wrong-digest",
            RecoveryPolicy {
                decision: ApprovalDecision::Approved,
                lease_s: 8.0 * 3600.0,
                approved_plan_digest: Some("bad".into()),
            },
            None,
        ),
        (
            "revoked-day5",
            RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 8.0 * 3600.0, approved_plan_digest: None },
            Some(5.0 * 86_400.0),
        ),
    ];
    for (name, policy, revoke_at_s) in cases {
        let report = match revoke_at_s {
            Some(revoke) => {
                simulate_recovery_with_revocation(&model, "potable-water", 30.0 * 86_400.0, &events, policy, revoke)
            }
            None => simulate_recovery_with_policy(&model, "potable-water", 30.0 * 86_400.0, &events, policy),
        }
        .unwrap();
        let executed = report.log.iter().any(|entry| !entry.repaired.is_empty());
        let blocked = report.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
        println!("{name}\t{executed}\t{:.0}\t{blocked}", report.survived_s / 86_400.0);
    }
    ExitCode::SUCCESS
}
