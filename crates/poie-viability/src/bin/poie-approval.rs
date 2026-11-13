// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S3 approval state and lease: required, rejected, approved, expired.

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
    let events = vec![
        FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p1".into() },
        FailureEvent { at_s: 20.0 * 86400.0, asset: "pump-p1".into() },
    ];
    println!("decision\tlease_h\tsurvived_days\tblocked_by");
    for (decision, lease_h) in [
        (ApprovalDecision::Approved, 8.0),
        (ApprovalDecision::Required, 4.0),
        (ApprovalDecision::Rejected, 4.0),
        (ApprovalDecision::Approved, 2.0),
    ] {
        let policy = RecoveryPolicy { decision, lease_s: lease_h * 3600.0 };
        let report = simulate_recovery_with_policy(&model, "potable-water", 30.0 * 86400.0, &events, policy).unwrap();
        let blocked = report.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
        println!("{decision:?}\t{lease_h:.1}\t{:.0}\t{blocked}", report.survived_s / 86400.0);
    }
    ExitCode::SUCCESS
}
