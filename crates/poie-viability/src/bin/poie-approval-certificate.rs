// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S4 certificate for the S3 approval gate.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{
    simulate_recovery_with_approval_schedule, simulate_recovery_with_denial, simulate_recovery_with_policy,
    simulate_recovery_with_revocation, ApprovalDecision, FailureEvent, RecoveryPolicy,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let events = vec![FailureEvent { at_s: 10.0 * 86_400.0, asset: "pump-p1".into() }];
    type Case<'a> = (&'a str, ApprovalDecision, f64, bool, Option<f64>, Option<f64>);
    let cases: Vec<Case> = vec![
        ("required", ApprovalDecision::Required, 8.0 * 3600.0, true, None, None),
        ("rejected", ApprovalDecision::Rejected, 8.0 * 3600.0, true, None, None),
        ("lease-1h", ApprovalDecision::Approved, 3600.0, true, None, None),
        ("approved-8h", ApprovalDecision::Approved, 8.0 * 3600.0, true, None, None),
        ("wrong-digest", ApprovalDecision::Approved, 8.0 * 3600.0, false, None, None),
        ("revoked-day5", ApprovalDecision::Approved, 8.0 * 3600.0, true, Some(5.0 * 86_400.0), None),
        ("denied-day10", ApprovalDecision::Approved, 8.0 * 3600.0, true, None, Some(10.0 * 86_400.0)),
    ];
    let mut rows = Vec::new();
    for (name, decision, lease_s, digest_ok, revoke_at_s, deny_at_s) in cases {
        let policy = RecoveryPolicy {
            decision,
            lease_s,
            approved_plan_digest: if digest_ok { None } else { Some("bad".into()) },
        };
        let report = match (revoke_at_s, deny_at_s) {
            (Some(revoke), _) => {
                simulate_recovery_with_revocation(&model, "potable-water", 30.0 * 86_400.0, &events, policy, revoke)
            }
            (None, Some(deny)) => {
                simulate_recovery_with_denial(&model, "potable-water", 30.0 * 86_400.0, &events, policy, deny)
            }
            (None, None) => simulate_recovery_with_policy(&model, "potable-water", 30.0 * 86_400.0, &events, policy),
        }
        .unwrap();
        let executed = report.log.iter().any(|entry| !entry.repaired.is_empty());
        let blocked = report.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
        rows.push(serde_json::json!({
            "name": name,
            "decision": format!("{decision:?}"),
            "lease_s": lease_s,
            "digest_ok": digest_ok,
            "revoke_at_s": revoke_at_s,
            "deny_at_s": deny_at_s,
            "executed": executed,
            "blocked_by": blocked,
        }));
    }
    let schedule_events = vec![
        FailureEvent { at_s: 10.0 * 86_400.0, asset: "pump-p1".into() },
        FailureEvent { at_s: 20.0 * 86_400.0, asset: "pump-p1".into() },
    ];
    let schedule_policy =
        RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 8.0 * 3600.0, approved_plan_digest: None };
    let approved_all = simulate_recovery_with_policy(
        &model,
        "potable-water",
        30.0 * 86_400.0,
        &schedule_events,
        schedule_policy.clone(),
    )
    .unwrap();
    let all_receipts = approved_all.log.iter().filter(|entry| entry.receipt.is_some()).count();
    let all_blocked =
        approved_all.log.iter().find_map(|entry| entry.blocked_by.clone()).unwrap_or_else(|| "none".into());
    let schedule = vec![(0.0, ApprovalDecision::Approved), (20.0 * 86_400.0, ApprovalDecision::Rejected)];
    let rejected = simulate_recovery_with_approval_schedule(
        &model,
        "potable-water",
        30.0 * 86_400.0,
        &schedule_events,
        schedule_policy,
        &schedule,
    )
    .unwrap();
    let rejected_receipts = rejected.log.iter().filter(|entry| entry.receipt.is_some()).count();
    let rejected_blocked =
        rejected.log.iter().find_map(|entry| entry.blocked_by.clone()).unwrap_or_else(|| "none".into());
    let schedule_rows = vec![
        serde_json::json!({
            "name": "approved-all",
            "receipts": all_receipts,
            "survived_days": approved_all.survived_s / 86_400.0,
            "blocked_by": all_blocked,
        }),
        serde_json::json!({
            "name": "reject-day20",
            "receipts": rejected_receipts,
            "survived_days": rejected.survived_s / 86_400.0,
            "blocked_by": rejected_blocked,
        }),
    ];
    let certificate = serde_json::json!({
        "schedule_cases": schedule_rows,
        "schema": "poie.approval-certificate/1",
        "plan_duration_s": 10800.0,
        "cases": rows,
        "claim_scope": "model/simulation approval semantics only; no physical authority claim",
    });
    println!("{}", serde_json::to_string_pretty(&certificate).expect("certificate JSON"));
    ExitCode::SUCCESS
}
