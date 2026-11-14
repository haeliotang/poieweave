// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S3 approval revocation behavior on Base-05.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_recovery_with_revocation, ApprovalDecision, FailureEvent, RecoveryPolicy};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-05/model.toml")).unwrap();
    let events = vec![FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p1".into() }];
    for (label, revoke_at_s) in [("revoke-day5", 5.0 * 86400.0), ("revoke-start", 0.0)] {
        let report = simulate_recovery_with_revocation(
            &model,
            "potable-water",
            30.0 * 86400.0,
            &events,
            RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 8.0 * 3600.0, approved_plan_digest: None },
            revoke_at_s,
        )
        .unwrap();
        let blocked = report.log.iter().find_map(|entry| entry.blocked_by.clone()).unwrap_or_default();
        println!("{label}: survived_days={:.1} blocked={blocked}", report.survived_s / 86400.0);
    }
    ExitCode::SUCCESS
}
