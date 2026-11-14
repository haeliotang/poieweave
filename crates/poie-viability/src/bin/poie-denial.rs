// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S3: a per-event denial after prior approvals.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{
    simulate_recovery_with_denial, simulate_recovery_with_policy, ApprovalDecision, FailureEvent, RecoveryPolicy,
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
    println!("deny_day\texecuted\tsurvived_days\tblocked_by");
    for deny_day in [None, Some(10.0), Some(20.0), Some(5.0)] {
        let report = match deny_day {
            Some(day) => simulate_recovery_with_denial(
                &model,
                "potable-water",
                30.0 * 86_400.0,
                &events,
                policy.clone(),
                day * 86_400.0,
            ),
            None => simulate_recovery_with_policy(&model, "potable-water", 30.0 * 86_400.0, &events, policy.clone()),
        }
        .unwrap();
        let executed = report.log.iter().any(|entry| !entry.repaired.is_empty());
        let blocked = report.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
        let label = deny_day.map(|day| format!("{day:.0}")).unwrap_or_else(|| "none".into());
        println!("{label}\t{executed}\t{:.0}\t{blocked}", report.survived_s / 86_400.0);
    }
    ExitCode::SUCCESS
}
