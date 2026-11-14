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
    for max_failures in [2, 3] {
        let report = joint_coverage_with_order_search(
            &model,
            "potable-water",
            12.0 * 3600.0,
            max_failures,
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        )
        .unwrap();
        println!(
            "max_failures={max_failures} recoverable_cases={} standard_joint={} best_joint={} order_sensitive={}",
            report.total,
            report.standard_joint,
            report.best_joint,
            report.order_sensitive(),
        );
    }
    ExitCode::SUCCESS
}
