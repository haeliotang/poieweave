// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 resupply sensitivity: one extra sleeve after the Base-06 joint conflict.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{Lot, LotQuality, Model};
use poie_viability::survival::{joint_coverage, BatteryLimits};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let mut model = Model::load(&root.join("scenarios/base-06/model.toml")).unwrap_or_else(|error| {
        panic!("load Base-06: {error}");
    });
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    let before = joint_coverage(&model, "potable-water", 12.0 * 3600.0, 2, battery).unwrap();
    model.lots.push(Lot {
        id: "resupply-sleeve".into(),
        resource: "sleeve-p1".into(),
        si: 1.0,
        quality: LotQuality::Accepted,
    });
    let after = joint_coverage(&model, "potable-water", 12.0 * 3600.0, 2, battery).unwrap();
    println!(
        "before\tindependent={}\tjoint={}\tgap={}",
        before.independent_recoverable,
        before.joint_recoverable,
        before.optimistic_gap()
    );
    println!(
        "after_resupply\tindependent={}\tjoint={}\tgap={}",
        after.independent_recoverable,
        after.joint_recoverable,
        after.optimistic_gap()
    );
    ExitCode::SUCCESS
}
