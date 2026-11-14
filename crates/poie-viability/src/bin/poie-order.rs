// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 repair-order sensitivity on Base-06.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{joint_coverage_with_order_search, BatteryLimits};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-06/model.toml")).unwrap();
    let report = joint_coverage_with_order_search(
        &model,
        "potable-water",
        12.0 * 3600.0,
        2,
        Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
    )
    .unwrap();
    println!("recoverable_cases={}", report.total);
    println!("standard_joint={}", report.standard_joint);
    println!("best_joint={}", report.best_joint);
    println!("order_sensitive={}", report.order_sensitive());
    for case in &report.improved {
        println!("improved={:?}", case.failed);
    }
    ExitCode::SUCCESS
}
