// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S4: export S3 approval cases for an independent Python gate check.

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
    let cases: Vec<(&str, ApprovalDecision, f64, bool, Option<f64>)> = vec![
        ("required", ApprovalDecision::Required, 8.0 * 3600.0, true, None),
        ("rejected", ApprovalDecision::Rejected, 8.0 * 3600.0, true, None),
        ("lease-1h", ApprovalDecision::Approved, 3600.0, true, None),
        ("approved-8h", ApprovalDecision::Approved, 8.0 * 3600.0, true, None),
        ("wrong-digest", ApprovalDecision::Approved, 8.0 * 3600.0, false, None),
        ("revoked-day5", ApprovalDecision::Approved, 8.0 * 3600.0, true, Some(5.0 * 86_400.0)),
    ];
    println!("{{\"plan_duration_s\":10800,\"cases\":[");
    for (index, (name, decision, lease_s, digest_ok, revoke_at_s)) in cases.iter().enumerate() {
        let policy = RecoveryPolicy {
            decision: *decision,
            lease_s: *lease_s,
            approved_plan_digest: if *digest_ok { None } else { Some("bad".into()) },
        };
        let report = match revoke_at_s {
            Some(revoke) => {
                simulate_recovery_with_revocation(&model, "potable-water", 30.0 * 86_400.0, &events, policy, *revoke)
            }
            None => simulate_recovery_with_policy(&model, "potable-water", 30.0 * 86_400.0, &events, policy),
        }
        .unwrap();
        let executed = report.log.iter().any(|entry| !entry.repaired.is_empty());
        let blocked = report.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
        let comma = if index + 1 == cases.len() { "" } else { "," };
        println!(
            "{{\"name\":\"{name}\",\"decision\":\"{decision:?}\",\"lease_s\":{lease_s},\"digest_ok\":{digest_ok},\"revoke_at_s\":{},\"executed\":{executed},\"blocked_by\":\"{}\"}}{comma}",
            revoke_at_s.map(|value| format!("{value:.0}")).unwrap_or_else(|| "null".into()),
            blocked.replace('"', "")
        );
    }
    println!("]}}");
    ExitCode::SUCCESS
}
