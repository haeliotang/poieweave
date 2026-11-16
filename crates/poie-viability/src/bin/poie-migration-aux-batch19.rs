// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase E sample: regeneration and resupply joint coverage.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{Lot, LotQuality, Model};
use poie_viability::engine::{run_engine, EngineOptions, EngineTermination};
use poie_viability::survival::{failure_envelope, joint_coverage, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn canonical_joint(model: &Model, service: &str, horizon_s: f64, max_failures: usize) -> (usize, usize) {
    let envelope = failure_envelope(model, service, horizon_s, max_failures).unwrap();
    let mut independent = 0;
    let mut joint = 0;
    for case in &envelope.cases {
        if case.recoverable {
            independent += 1;
            let events: Vec<FailureEvent> =
                case.failed.iter().map(|asset| FailureEvent { at_s: 0.0, asset: asset.clone() }).collect();
            let options = EngineOptions {
                services: vec![service],
                horizon_s,
                failures: events,
                resupplies: Vec::new(),
                resources: vec!["robot-m1".into(), "lathe-1".into()],
                alternative_pool: vec!["robot-m1".into()],
                battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
                load_kw: 0.0,
            };
            if run_engine(model, &options).unwrap().termination == EngineTermination::CompletedHorizon {
                joint += 1;
            }
        }
    }
    (independent, joint)
}

fn main() -> ExitCode {
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!(
        "experiment\trow\tlegacy_independent\tlegacy_joint\tcanonical_independent\tcanonical_joint\tclassification"
    );
    for model_name in ["base-06", "base-07"] {
        let model = Model::load(&root().join(format!("scenarios/{model_name}/model.toml"))).unwrap();
        let legacy = joint_coverage(&model, "potable-water", 24.0 * 3600.0, 2, battery).unwrap();
        let (independent, joint) = canonical_joint(&model, "potable-water", 24.0 * 3600.0, 2);
        let class = if legacy.independent_recoverable == independent && legacy.joint_recoverable == joint {
            "EXPECTED_AGREEMENT"
        } else if legacy.independent_recoverable == independent {
            "SEMANTIC_CORRECTION_JOINT_COVERAGE"
        } else {
            "UNEXPLAINED"
        };
        println!(
            "regeneration\t{model_name}\t{}\t{}\t{independent}\t{joint}\t{class}",
            legacy.independent_recoverable, legacy.joint_recoverable
        );
    }
    let mut model = Model::load(&root().join("scenarios/base-06/model.toml")).unwrap();
    let before = joint_coverage(&model, "potable-water", 12.0 * 3600.0, 2, battery).unwrap();
    let before_canonical = canonical_joint(&model, "potable-water", 12.0 * 3600.0, 2);
    let before_class =
        if before.independent_recoverable == before_canonical.0 && before.joint_recoverable == before_canonical.1 {
            "EXPECTED_AGREEMENT"
        } else {
            "SEMANTIC_CORRECTION_JOINT_COVERAGE"
        };
    println!(
        "resupply\tbefore\t{}\t{}\t{}\t{}\t{before_class}",
        before.independent_recoverable, before.joint_recoverable, before_canonical.0, before_canonical.1
    );
    model.lots.push(Lot {
        id: "resupply-sleeve".into(),
        resource: "sleeve-p1".into(),
        si: 1.0,
        quality: LotQuality::Accepted,
    });
    let after = joint_coverage(&model, "potable-water", 12.0 * 3600.0, 2, battery).unwrap();
    let after_canonical = canonical_joint(&model, "potable-water", 12.0 * 3600.0, 2);
    let after_class =
        if after.independent_recoverable == after_canonical.0 && after.joint_recoverable == after_canonical.1 {
            "EXPECTED_AGREEMENT"
        } else {
            "SEMANTIC_CORRECTION_JOINT_COVERAGE"
        };
    println!(
        "resupply\tafter_resupply\t{}\t{}\t{}\t{}\t{after_class}",
        after.independent_recoverable, after.joint_recoverable, after_canonical.0, after_canonical.1
    );
    ExitCode::SUCCESS
}
