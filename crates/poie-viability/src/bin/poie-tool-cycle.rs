// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S1: repairing the tool that produces its own repair part is a cycle unless a
//! spare part is stocked.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{AssetStatus, Model};
use poie_viability::closure::{analyze, BlockReason, ClosureOutcome};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn reason(reason: &BlockReason) -> String {
    match reason {
        BlockReason::Cycle { kind, id } => format!("cycle:{kind}:{id}"),
        BlockReason::NoRestorer(asset) => format!("no_restorer:{asset}"),
        BlockReason::MissingResource { resource, .. } => format!("missing_resource:{resource}"),
        BlockReason::MissingCapability(capability) => format!("missing_capability:{capability}"),
        other => format!("{other:?}"),
    }
}

fn main() -> ExitCode {
    let root = root();
    let base = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    println!("case\tnut_stock\toutcome\tdetail");
    for (case, stock) in [("no-spare", 0.0), ("one-spare", 1.0)] {
        let mut model = base.clone();
        model.assets.get_mut("lathe-1").unwrap().status = AssetStatus::Failed;
        model.lots.iter_mut().find(|lot| lot.id == "lot-lathe-nut").unwrap().si = stock;
        match analyze(&model, "lathe-1", 24.0 * 3600.0) {
            ClosureOutcome::Restorable(plan) => {
                println!("{case}\t{stock:.0}\trestorable\t{:.0}s", plan.total_duration_s);
            }
            ClosureOutcome::Blocked(blocked) => {
                println!("{case}\t{stock:.0}\tblocked\t{}", reason(&blocked.reasons[0]));
            }
        }
    }
    ExitCode::SUCCESS
}
