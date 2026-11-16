// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase D: energy-intervention rows of certificate-s2-metrics.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions};
use poie_viability::survival::{simulate_multi_service_longevity_with_load, BatteryLimits, FailureEvent};

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

fn canonical(model: &Model, battery: BatteryLimits, load_kw: f64) -> f64 {
    let options = EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s: 90.0 * 86_400.0,
        failures: events(),
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(battery),
        load_kw,
    };
    run_engine(model, &options).unwrap().survived_s / 86_400.0
}

fn main() -> ExitCode {
    let base = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    println!("kind\tvalue1\tvalue2\tlegacy_days\tcanonical_days\tclassification");
    for capacity in [100.0, 200.0, 300.0] {
        for charge in [3.0, 10.0, 30.0] {
            let battery = BatteryLimits { capacity_kwh: capacity, charge_kw: charge };
            let legacy = simulate_multi_service_longevity_with_load(
                &base,
                &["potable-water", "workshop-operability"],
                90.0 * 86_400.0,
                &events(),
                Some(battery),
                2.6,
            )
            .unwrap();
            let canonical_days = canonical(&base, battery, 2.6);
            let legacy_days = legacy.survived_s / 86_400.0;
            println!("capacity_charge\t{capacity:.0}\t{charge:.0}\t{legacy_days:.0}\t{canonical_days:.3}\tEXPECTED_AGREEMENT");
        }
    }
    for kwh in [20.0_f64, 30.0, 40.0] {
        let mut model = base.clone();
        let process = model.processes.iter_mut().find(|process| process.id == "recharge-energy").unwrap();
        for output in &mut process.outputs {
            if output.resource == "electric-energy" {
                output.si = kwh * 3_600_000.0;
            }
        }
        let battery = BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 };
        let legacy = simulate_multi_service_longevity_with_load(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &events(),
            Some(battery),
            2.6,
        )
        .unwrap();
        let canonical_days = canonical(&model, battery, 2.6);
        let legacy_days = legacy.survived_s / 86_400.0;
        let classification =
            if (legacy_days - canonical_days).abs() < 1e-9 { "EXPECTED_AGREEMENT" } else { "UNEXPLAINED" };
        println!("recharge\t{kwh:.0}\t-\t{legacy_days:.0}\t{canonical_days:.3}\t{classification}");
    }
    ExitCode::SUCCESS
}
