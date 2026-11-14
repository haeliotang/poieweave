// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S3 multi-service recovery loop on Base-05.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{
    simulate_multi_service_recovery_with_policy, ApprovalDecision, FailureEvent, RecoveryPolicy,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-05/model.toml")).unwrap();
    let one_event = vec![FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p1".into() }];
    let two_events = vec![
        FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p1".into() },
        FailureEvent { at_s: 20.0 * 86400.0, asset: "lathe-1".into() },
    ];
    for (label, lease_s, events) in [
        ("approved-8h", 8.0 * 3600.0, one_event.as_slice()),
        ("lease-1h", 3600.0, one_event.as_slice()),
        ("two-events-8h", 8.0 * 3600.0, two_events.as_slice()),
    ] {
        let report = simulate_multi_service_recovery_with_policy(
            &model,
            &["potable-water", "workshop-operability"],
            30.0 * 86400.0,
            events,
            RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s, approved_plan_digest: None },
        )
        .unwrap();
        let receipts = report.log.iter().filter(|entry| entry.receipt.is_some()).count();
        let blocked = report.log.iter().find_map(|entry| entry.blocked_by.clone()).unwrap_or_default();
        println!(
            "{label}: services={} survived_days={:.1} receipts={receipts} blocked={blocked}",
            report.services.join(","),
            report.survived_s / 86400.0,
        );
    }
    ExitCode::SUCCESS
}
