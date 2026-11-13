// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S1 recursive capability-closure runner.
//!
//! Prints a deterministic table for the Base-01/Base-04 experiments. The
//! output is replayed by CI; the report explains the assumptions.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{AssetStatus, Model};
use poie_viability::closure::{analyze, BlockReason, ClosureOutcome, StepAction};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn model(root: &Path, relative: &str) -> Model {
    Model::load(&root.join(relative)).unwrap_or_else(|error| panic!("load {relative}: {error}"))
}

fn without_lot(mut model: Model, lot: &str) -> Model {
    model.lots.retain(|item| item.id != lot);
    model
}

fn with_status(mut model: Model, asset: &str, status: AssetStatus) -> Model {
    model.assets.get_mut(asset).expect("asset exists").status = status;
    model
}

fn step_label(action: &StepAction) -> String {
    match action {
        StepAction::Restore(asset) => format!("restore:{asset}"),
        StepAction::Produce(resource) => format!("produce:{resource}"),
    }
}

fn reason_label(reason: &BlockReason) -> String {
    match reason {
        BlockReason::UnknownAsset(asset) => format!("unknown_asset:{asset}"),
        BlockReason::NoRestorer(asset) => format!("no_restorer:{asset}"),
        BlockReason::MissingResource { resource, .. } => format!("missing_resource:{resource}"),
        BlockReason::MissingCapability(capability) => format!("missing_capability:{capability}"),
        BlockReason::Cycle { kind, id } => format!("cycle:{kind}:{id}"),
        BlockReason::UnknownDuration(process) => format!("unknown_duration:{process}"),
        BlockReason::HorizonExceeded { .. } => "horizon_exceeded".to_string(),
        BlockReason::UnverifiedProcess { process, .. } => format!("unverified_process:{process}"),
    }
}

fn run_case(name: &str, scenario: &str, target: &str, horizon_s: f64, model: &Model) {
    match analyze(model, target, horizon_s) {
        ClosureOutcome::Restorable(plan) => {
            let steps = plan
                .steps
                .iter()
                .map(|step| format!("{}={}", step_label(&step.action), step.process))
                .collect::<Vec<_>>()
                .join("|");
            println!("{name}\t{scenario}\t{target}\tRESTORABLE\tsteps={steps}\ttotal_s={:.0}", plan.total_duration_s);
        }
        ClosureOutcome::Blocked(blocked) => {
            let reasons = blocked.reasons.iter().map(reason_label).collect::<Vec<_>>().join("|");
            println!("{name}\t{scenario}\t{target}\tBLOCKED\treasons={reasons}");
        }
    }
}

fn main() -> ExitCode {
    let root = root();
    let base01 = model(&root, "scenarios/base-01/model.toml");
    let base04 = model(&root, "scenarios/base-04/model.toml");
    let base04_lathe_failed = with_status(base04.clone(), "lathe-1", AssetStatus::Failed);
    let base01_lathe_failed = with_status(base01.clone(), "lathe-1", AssetStatus::Failed);
    let base04_lathe_no_spare =
        with_status(without_lot(base04.clone(), "lot-lathe-nut"), "lathe-1", AssetStatus::Failed);
    let base01_no_sleeve = without_lot(base01.clone(), "lot-sleeve-01");
    let base01_no_breaker = with_status(without_lot(base01.clone(), "lot-breaker"), "bus-1", AssetStatus::Failed);

    println!("case\tscenario\ttarget\tverdict\tdetail");
    run_case("s1-base04-lathe", "base-04", "lathe-1", 12.0 * 3600.0, &base04_lathe_failed);
    run_case("s1-base04-pump", "base-04", "pump-p1", 12.0 * 3600.0, &base04);
    run_case("s1-base04-pump-too-short", "base-04", "pump-p1", 2.0 * 3600.0, &base04);
    run_case("s1-base01-lathe", "base-01", "lathe-1", 12.0 * 3600.0, &base01_lathe_failed);
    run_case("s1-base04-lathe-no-spare", "base-04", "lathe-1", 12.0 * 3600.0, &base04_lathe_no_spare);
    run_case("s1-base01-pump-no-sleeve", "base-01", "pump-p1", 12.0 * 3600.0, &base01_no_sleeve);
    run_case("s1-base01-bus-no-breaker", "base-01", "bus-1", 12.0 * 3600.0, &base01_no_breaker);
    ExitCode::SUCCESS
}
