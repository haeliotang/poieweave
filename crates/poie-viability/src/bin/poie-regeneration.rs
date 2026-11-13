// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2/B regeneration sensitivity: local bronze-swarf recycling versus Base-06.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{joint_coverage, BatteryLimits};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    for model_name in ["base-06", "base-07"] {
        let model = Model::load(&root.join(format!("scenarios/{model_name}/model.toml")))
            .unwrap_or_else(|error| panic!("load {model_name}: {error}"));
        let report = joint_coverage(&model, "potable-water", 24.0 * 3600.0, 2, battery).unwrap();
        println!(
            "{model_name}\tindependent={}\tjoint={}\tgap={}",
            report.independent_recoverable,
            report.joint_recoverable,
            report.optimistic_gap()
        );
    }
    ExitCode::SUCCESS
}
