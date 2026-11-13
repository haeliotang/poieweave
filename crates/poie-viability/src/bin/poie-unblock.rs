// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Counterfactual: minimal bronze addition that unblocks Base-09 pump repair.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{AssetStatus, Lot, LotQuality, Model};
use poie_viability::closure::{analyze, ClosureOutcome};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let base = Model::load(&root.join("scenarios/base-09/model.toml")).unwrap_or_else(|error| {
        panic!("load Base-09: {error}");
    });
    println!("bronze_kg\toutcome");
    for quantity_kg in [1.1_f64, 1.2_f64, 1.5_f64] {
        let mut scenario = base.clone();
        scenario.lots.push(Lot {
            id: format!("counterfactual-bronze-{quantity_kg}"),
            resource: "bronze-bar".into(),
            si: quantity_kg,
            quality: LotQuality::Accepted,
        });
        if let Some(asset) = scenario.assets.get_mut("pump-p1") {
            asset.status = AssetStatus::Failed;
        }
        let outcome = match analyze(&scenario, "pump-p1", 24.0 * 3600.0) {
            ClosureOutcome::Restorable(_) => "RESTORABLE",
            ClosureOutcome::Blocked(_) => "BLOCKED",
        };
        println!("{quantity_kg:.1}\t{outcome}");
    }
    ExitCode::SUCCESS
}
