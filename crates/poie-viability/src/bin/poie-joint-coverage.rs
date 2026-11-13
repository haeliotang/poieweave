// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 all-pairs independent vs joint recovery coverage on Base-06.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{joint_coverage, BatteryLimits};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-06/model.toml")).unwrap_or_else(|error| {
        panic!("load Base-06: {error}");
    });
    let report = joint_coverage(
        &model,
        "potable-water",
        12.0 * 3600.0,
        2,
        Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
    )
    .unwrap();
    println!(
        "service={} horizon_s={:.0} total={} independent={} joint={} optimistic_gap={}",
        report.service,
        report.horizon_s,
        report.total(),
        report.independent_recoverable,
        report.joint_recoverable,
        report.optimistic_gap()
    );
    for case in &report.cases {
        if case.independent_recoverable && !case.joint_recoverable {
            println!("gap\tfailed={}\tjoint=BLOCKED", case.failed.join(","));
        }
    }
    ExitCode::SUCCESS
}
