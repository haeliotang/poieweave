// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase E sample: resupply-time, resupply-delivery and regen-cross.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::engine::{run_engine, EngineOptions, EngineTermination};
use poie_viability::survival::{
    simulate_multi_service_longevity_with_limits, simulate_multi_service_longevity_with_resupply, BatteryLimits,
    FailureEvent, Resupply,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn horizon_events(interval: u32) -> Vec<FailureEvent> {
    let mut events: Vec<FailureEvent> = (1..=(89 / interval))
        .map(|index| FailureEvent { at_s: f64::from(index * interval) * 86_400.0, asset: "pump-p1".into() })
        .collect();
    events.push(FailureEvent { at_s: 35.0 * 86_400.0, asset: "lathe-1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    events
}

fn canonical(model: &Model, events: Vec<FailureEvent>, resupplies: Vec<Resupply>, load_kw: f64) -> (f64, String) {
    let options = EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s: 90.0 * 86_400.0,
        failures: events,
        resupplies,
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

fn main() -> ExitCode {
    let base = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    println!("experiment\trow\tlegacy_days\tcanonical_days\tclassification");

    let mut time_model = base.clone();
    time_model.lots.iter_mut().find(|lot| lot.id == "lot-insert").unwrap().si = 1.0;
    for interval in [10_u32, 20, 30, 60] {
        let mut resupplies = Vec::new();
        let mut day = f64::from(interval);
        while day < 90.0 {
            resupplies.push(Resupply { at_s: day * 86_400.0, resource: "cutting-insert".into(), si: 1.0 });
            day += f64::from(interval);
        }
        let events = (1..=8)
            .map(|index| FailureEvent { at_s: f64::from(index * 10) * 86_400.0, asset: "pump-p1".into() })
            .collect::<Vec<_>>();
        let legacy = simulate_multi_service_longevity_with_resupply(
            &time_model,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &events,
            &resupplies,
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
            0.0,
        )
        .unwrap();
        let (canonical_days, _) = canonical(&time_model, events, resupplies, 0.0);
        let legacy_days = legacy.survived_s / 86_400.0;
        let class = if (legacy_days - canonical_days).abs() < 1e-9 { "EXPECTED_AGREEMENT" } else { "UNEXPLAINED" };
        println!("resupply-time\tinterval={interval}\t{legacy_days:.0}\t{canonical_days:.3}\t{class}");
    }

    for (label, stock, resupply) in
        [("one-no-resupply", 1.0, false), ("four-no-resupply", 4.0, false), ("one-resupply", 1.0, true)]
    {
        let mut model = base.clone();
        model.lots.iter_mut().find(|lot| lot.id == "lot-insert").unwrap().si = stock;
        let events = (1..=8)
            .map(|index| FailureEvent { at_s: f64::from(index * 10) * 86_400.0, asset: "pump-p1".into() })
            .collect::<Vec<_>>();
        let supplies = if resupply {
            [30.0, 60.0]
                .into_iter()
                .map(|day| Resupply { at_s: day * 86_400.0, resource: "cutting-insert".into(), si: 1.0 })
                .collect()
        } else {
            Vec::new()
        };
        let legacy = simulate_multi_service_longevity_with_resupply(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &events,
            &supplies,
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
            0.0,
        )
        .unwrap();
        let (canonical_days, _) = canonical(&model, events, supplies, 0.0);
        let legacy_days = legacy.survived_s / 86_400.0;
        let class = if (legacy_days - canonical_days).abs() < 1e-9 { "EXPECTED_AGREEMENT" } else { "UNEXPLAINED" };
        println!("resupply-delivery\t{label}\t{legacy_days:.0}\t{canonical_days:.3}\t{class}");
    }

    for fraction in [1.0_f64, 0.75, 0.5] {
        let mut model = base.clone();
        let process = model.processes.iter_mut().find(|process| process.id == "recycle-bronze").unwrap();
        for output in &mut process.outputs {
            if output.resource == "bronze-bar" {
                output.si *= fraction;
            }
        }
        for interval in [10_u32, 15, 20, 30] {
            let events = horizon_events(interval);
            let legacy = simulate_multi_service_longevity_with_limits(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86_400.0,
                &events,
                Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
            )
            .unwrap();
            let (canonical_days, _) = canonical(&model, events, Vec::new(), 0.0);
            let legacy_days = legacy.survived_s / 86_400.0;
            let class = if (legacy_days - canonical_days).abs() < 1e-9 {
                "EXPECTED_AGREEMENT"
            } else if legacy_days.round() == canonical_days.round() {
                "SEMANTIC_CORRECTION"
            } else {
                "UNEXPLAINED"
            };
            println!(
                "regen-cross\tyield={fraction:.2},interval={interval}\t{legacy_days:.0}\t{canonical_days:.3}\t{class}"
            );
        }
    }
    ExitCode::SUCCESS
}
