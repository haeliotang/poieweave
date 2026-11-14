// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S3: an event inside a repair window is not an interruption in the model.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::closure::{analyze, ClosureOutcome};
use poie_viability::survival::{simulate_recovery_with_policy, ApprovalDecision, FailureEvent, RecoveryPolicy};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let mut model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    model.assets.get_mut("pump-p1").unwrap().status = poie_ontology::model::AssetStatus::Failed;
    let repair_s = match analyze(&model, "pump-p1", 24.0 * 3600.0) {
        ClosureOutcome::Restorable(plan) => plan.total_duration_s,
        ClosureOutcome::Blocked(_) => 0.0,
    };
    let event_s = 3600.0;
    let policy =
        RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 8.0 * 3600.0, approved_plan_digest: None };
    let events = vec![FailureEvent { at_s: event_s, asset: "filter-1".into() }];
    let report = simulate_recovery_with_policy(&model, "potable-water", 86_400.0, &events, policy).unwrap();
    let receipts = report.log.iter().filter(|entry| entry.receipt.is_some()).count();
    let blocked = report.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
    println!("repair_s\tevent_s\tinside_repair\treceipts\tblocked_by");
    println!("{repair_s:.0}\t{event_s:.0}\t{}\t{receipts}\t{blocked}", event_s > 0.0 && event_s < repair_s);
    ExitCode::SUCCESS
}
