// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase B: C6 Base-06 all-pairs joint coverage, legacy vs canonical.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions, EngineTermination};
use poie_viability::survival::{failure_envelope, joint_coverage, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let model = Model::load(&root().join("scenarios/base-06/model.toml")).unwrap();
    let horizon_s = 12.0 * 3600.0;
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    let legacy = joint_coverage(&model, "potable-water", horizon_s, 2, battery).unwrap();
    let envelope = failure_envelope(&model, "potable-water", horizon_s, 2).unwrap();
    let legacy_map: BTreeMap<Vec<String>, bool> =
        legacy.cases.iter().map(|case| (case.failed.clone(), case.joint_recoverable)).collect();
    let mut canonical_joint = 0;
    let mut first_divergence = "none".to_string();
    let mut canonical_map = BTreeMap::new();
    let mut divergences: Vec<(String, bool, bool, String)> = Vec::new();
    for case in &envelope.cases {
        if !case.recoverable {
            continue;
        }
        let events: Vec<FailureEvent> =
            case.failed.iter().map(|asset| FailureEvent { at_s: 0.0, asset: asset.clone() }).collect();
        let options = EngineOptions {
            services: vec!["potable-water"],
            horizon_s,
            failures: events,
            resupplies: Vec::new(),
            resources: vec!["robot-m1".into(), "lathe-1".into()],
            alternative_pool: vec!["robot-m1".into()],
            battery,
            load_kw: 0.0,
        };
        let outcome = run_engine(&model, &options).unwrap();
        let joint = outcome.termination == EngineTermination::CompletedHorizon;
        if joint {
            canonical_joint += 1;
        }
        canonical_map.insert(case.failed.clone(), joint);
        if let Some(legacy_joint) = legacy_map.get(&case.failed) {
            if *legacy_joint != joint {
                let reason = match &outcome.termination {
                    EngineTermination::Blocked(reason) => reason.clone(),
                    EngineTermination::IncompleteAtHorizon(reason) => format!("incomplete:{reason}"),
                    EngineTermination::CompletedHorizon => "completed".into(),
                };
                divergences.push((case.failed.join(","), *legacy_joint, joint, reason));
                if first_divergence == "none" {
                    first_divergence = case.failed.join(",");
                }
            }
        }
    }
    let classification = if legacy.joint_recoverable == canonical_joint
        && canonical_map.iter().all(|(failed, joint)| legacy_map.get(failed) == Some(joint))
    {
        "EXPECTED_AGREEMENT"
    } else {
        "UNEXPLAINED"
    };
    println!("service\ttotal\tindependent\tlegacy_joint\tcanonical_joint\tfirst_divergence\tclassification");
    println!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}",
        legacy.service,
        legacy.total(),
        legacy.independent_recoverable,
        legacy.joint_recoverable,
        canonical_joint,
        first_divergence,
        classification
    );
    println!("case\tlegacy_joint\tcanonical_joint\tcanonical_reason");
    for (case, legacy_joint, canonical_joint, reason) in &divergences {
        println!("{case}\t{legacy_joint}\t{canonical_joint}\t{reason}");
    }
    ExitCode::SUCCESS
}
