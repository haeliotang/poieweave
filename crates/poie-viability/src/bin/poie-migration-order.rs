// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase D: Base-06 repair-order negative result under canonical semantics.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions, EngineTermination};
use poie_viability::survival::{failure_envelope, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn permutations(items: &[String]) -> Vec<Vec<String>> {
    if items.len() <= 1 {
        return vec![items.to_vec()];
    }
    let mut result = Vec::new();
    for index in 0..items.len() {
        let mut rest = items.to_vec();
        let item = rest.remove(index);
        for mut tail in permutations(&rest) {
            let mut next = vec![item.clone()];
            next.append(&mut tail);
            result.push(next);
        }
    }
    result
}

fn recovers(model: &Model, order: &[String], horizon_s: f64) -> bool {
    let events: Vec<FailureEvent> =
        order.iter().map(|asset| FailureEvent { at_s: 0.0, asset: asset.clone() }).collect();
    let options = EngineOptions {
        services: vec!["potable-water"],
        horizon_s,
        failures: events,
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        load_kw: 0.0,
    };
    run_engine(model, &options).unwrap().termination == EngineTermination::CompletedHorizon
}

fn main() -> ExitCode {
    let model = Model::load(&root().join("scenarios/base-06/model.toml")).unwrap();
    let horizon_s = 12.0 * 3600.0;
    println!("max_failures\trecoverable_cases\tlegacy_standard\tcanonical_standard\tlegacy_best\tcanonical_best\torder_sensitive\tclassification");
    for (max_failures, legacy_standard, legacy_best) in [(2usize, 24usize, 24usize), (3, 40, 40)] {
        let envelope = failure_envelope(&model, "potable-water", horizon_s, max_failures).unwrap();
        let mut recoverable = 0;
        let mut standard = 0;
        let mut best = 0;
        let mut improved = 0;
        for case in &envelope.cases {
            if !case.recoverable {
                continue;
            }
            recoverable += 1;
            let standard_ok = recovers(&model, &case.failed, horizon_s);
            if standard_ok {
                standard += 1;
            }
            let mut best_ok = false;
            for order in permutations(&case.failed) {
                if recovers(&model, &order, horizon_s) {
                    best_ok = true;
                    break;
                }
            }
            if best_ok {
                best += 1;
            }
            if best_ok && !standard_ok {
                improved += 1;
            }
        }
        let classification = if standard == legacy_standard && best == legacy_best && improved == 0 {
            "EXPECTED_AGREEMENT"
        } else if standard == legacy_standard {
            "SEMANTIC_CORRECTION_ORDER"
        } else {
            "UNEXPLAINED"
        };
        println!("{max_failures}\t{recoverable}\t{legacy_standard}\t{standard}\t{legacy_best}\t{best}\t{improved}\t{classification}");
    }
    ExitCode::SUCCESS
}
