// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase E sample: redundancy, long-horizon and recharge-rule.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{Asset, AssetStatus, Model};
use poie_viability::engine::{run_engine, EngineOptions, EngineTermination};
use poie_viability::survival::{
    simulate_multi_service_longevity_with_limits, simulate_multi_service_longevity_with_load, BatteryLimits,
    FailureEvent,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn events() -> Vec<FailureEvent> {
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86_400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86_400.0, asset: "lathe-1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    events
}

fn canonical(model: &Model, load_kw: f64) -> (f64, String) {
    let options = EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s: 90.0 * 86_400.0,
        failures: events(),
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        load_kw,
    };
    let outcome = run_engine(model, &options).unwrap();
    let termination = match &outcome.termination {
        EngineTermination::CompletedHorizon => "completed".to_string(),
        EngineTermination::Blocked(reason) => format!("blocked:{reason}"),
        EngineTermination::IncompleteAtHorizon(reason) => format!("incomplete:{reason}"),
    };
    (outcome.survived_s / 86_400.0, termination)
}

fn class(legacy: f64, canonical: f64) -> &'static str {
    if (legacy - canonical).abs() < 1e-9 {
        "EXPECTED_AGREEMENT"
    } else if legacy.round() == canonical.round() {
        "SEMANTIC_CORRECTION"
    } else {
        "UNEXPLAINED"
    }
}

fn redundant_pump(model: &mut Model) {
    model.assets.insert(
        "pump-p2".into(),
        Asset {
            id: "pump-p2".into(),
            kind: "pump".into(),
            status: AssetStatus::Available,
            status_evidence: None,
            requires: Vec::new(),
            requires_any: Vec::new(),
            provides: Vec::new(),
            capability_verification: BTreeMap::new(),
            degraded: None,
        },
    );
    let service = model.services.get_mut("potable-water").unwrap();
    service.requires.retain(|asset| asset != "pump-p1");
    service.requires_any.push("pump-p1".into());
    service.requires_any.push("pump-p2".into());
}

fn main() -> ExitCode {
    let base08 = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("experiment\trow\tlegacy_days\tcanonical_days\tclassification");

    for mode in ["baseline", "redundant-pump"] {
        let mut model = base08.clone();
        if mode == "redundant-pump" {
            redundant_pump(&mut model);
        }
        let legacy = simulate_multi_service_longevity_with_limits(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &events(),
            battery,
        )
        .unwrap();
        let (canonical_days, _) = canonical(&model, 0.0);
        let legacy_days = legacy.survived_s / 86_400.0;
        println!("redundancy\t{mode}\t{legacy_days:.0}\t{canonical_days:.3}\t{}", class(legacy_days, canonical_days));
    }

    let base05 = Model::load(&root().join("scenarios/base-05/model.toml")).unwrap();
    let legacy = simulate_multi_service_longevity_with_limits(
        &base05,
        &["potable-water", "workshop-operability"],
        90.0 * 86_400.0,
        &events(),
        battery,
    )
    .unwrap();
    let (canonical_days, _) = canonical(&base05, 0.0);
    let legacy_days = legacy.survived_s / 86_400.0;
    println!("long-horizon\thorizon\t{legacy_days:.0}\t{canonical_days:.3}\t{}", class(legacy_days, canonical_days));

    for (label, base) in [("recharge-rule-base08", &base08), ("recharge-rule-base05", &base05)] {
        for kwh in [10.0_f64, 20.0, 40.0] {
            let mut model = base.clone();
            let process = model.processes.iter_mut().find(|process| process.id == "recharge-energy").unwrap();
            for output in &mut process.outputs {
                if output.resource == "electric-energy" {
                    output.si = kwh * 3_600_000.0;
                }
            }
            let mut legacy_threshold = 0.0;
            let mut canonical_threshold = 0.0;
            for tenths in 1..=80 {
                let load_kw = f64::from(tenths) / 10.0;
                let legacy_report = simulate_multi_service_longevity_with_load(
                    &model,
                    &["potable-water", "workshop-operability"],
                    90.0 * 86_400.0,
                    &events(),
                    battery,
                    load_kw,
                )
                .unwrap();
                if legacy_threshold == 0.0
                    && legacy_report.log.iter().any(|entry| entry.blocked_by.as_deref() == Some("energy_depleted"))
                {
                    legacy_threshold = load_kw;
                }
                let (_, termination) = canonical(&model, load_kw);
                if canonical_threshold == 0.0 && termination.contains("energy_depleted") {
                    canonical_threshold = load_kw;
                }
            }
            let classification = if (legacy_threshold - canonical_threshold).abs() < 1e-9 {
                "EXPECTED_AGREEMENT"
            } else {
                "UNEXPLAINED"
            };
            println!("{label}\tkwh={kwh:.0}\t{legacy_threshold:.1}\t{canonical_threshold:.1}\t{classification}");
        }
    }
    ExitCode::SUCCESS
}
