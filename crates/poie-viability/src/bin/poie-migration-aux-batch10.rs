// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase E sample: joint-three, joint-worn and joint-yield.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{Flow, Lot, LotQuality, Model, Process, Verification, YieldSpec};
use poie_viability::engine::{run_engine, EngineOptions};
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sharpen_process(fraction: Option<f64>) -> Process {
    Process {
        id: "sharpen-insert".into(),
        inputs: vec![
            Flow { resource: "worn-insert".into(), si: 1.0 },
            Flow { resource: "electric-energy".into(), si: 0.3 * 3_600_000.0 },
        ],
        outputs: vec![Flow { resource: "cutting-insert".into(), si: 1.0 }],
        occupies: vec!["gauge-1".into()],
        requires_capabilities: Vec::new(),
        restores: Vec::new(),
        duration_s: Some(3600.0),
        verification: Verification::Modeled,
        asset_mass_change_kg: 0.0,
        mass_tolerance: 0.01,
        process_yield: fraction.map(|fraction| YieldSpec {
            output: "cutting-insert".into(),
            fraction,
            reject: "worn-insert-reject".into(),
        }),
    }
}

fn events() -> Vec<FailureEvent> {
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86_400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86_400.0, asset: "lathe-1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    events
}

fn canonical(model: &Model) -> f64 {
    let options = EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s: 90.0 * 86_400.0,
        failures: events(),
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        load_kw: 0.0,
    };
    run_engine(model, &options).unwrap().survived_s / 86_400.0
}

fn main() -> ExitCode {
    let base = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("experiment\trow\tlegacy_days\tcanonical_days\tclassification");
    for bronze in [3.0_f64, 6.0] {
        for energy in [20.0_f64, 40.0, 80.0] {
            let mut model = base.clone();
            model.add_process(sharpen_process(None));
            if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "cutting-insert") {
                lot.si = 2.0;
            }
            if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "bronze-bar") {
                lot.si = bronze;
            }
            if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "electric-energy") {
                lot.si = energy * 3.6e6;
            }
            let legacy = simulate_multi_service_longevity_with_limits(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86_400.0,
                &events(),
                battery,
            )
            .unwrap();
            let canonical_days = canonical(&model);
            let legacy_days = legacy.survived_s / 86_400.0;
            let class =
                if legacy_days.round() == canonical_days.round() { "EXPECTED_AGREEMENT" } else { "UNEXPLAINED" };
            println!(
                "joint-three\tbronze={bronze:.1},energy={energy:.0}\t{legacy_days:.0}\t{canonical_days:.3}\t{class}"
            );
        }
    }
    for worn in [0.0_f64, 2.0, 4.0] {
        for bronze in [3.0_f64, 6.0] {
            let mut model = base.clone();
            model.add_process(sharpen_process(Some(0.5)));
            if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "cutting-insert") {
                lot.si = 2.0;
            }
            if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "bronze-bar") {
                lot.si = bronze;
            }
            if worn > 0.0 {
                model.lots.push(Lot {
                    id: "lot-worn".into(),
                    resource: "worn-insert".into(),
                    si: worn,
                    quality: LotQuality::Accepted,
                });
            }
            let legacy = simulate_multi_service_longevity_with_limits(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86_400.0,
                &events(),
                battery,
            )
            .unwrap();
            let canonical_days = canonical(&model);
            let legacy_days = legacy.survived_s / 86_400.0;
            let class =
                if legacy_days.round() == canonical_days.round() { "EXPECTED_AGREEMENT" } else { "UNEXPLAINED" };
            println!("joint-worn\tworn={worn:.0},bronze={bronze:.1}\t{legacy_days:.0}\t{canonical_days:.3}\t{class}");
        }
    }
    for fraction in [1.0_f64, 0.8, 0.5] {
        for bronze in [3.0_f64, 6.0] {
            let mut model = base.clone();
            model.add_process(sharpen_process(if fraction >= 1.0 { None } else { Some(fraction) }));
            if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "cutting-insert") {
                lot.si = 2.0;
            }
            if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "bronze-bar") {
                lot.si = bronze;
            }
            let legacy = simulate_multi_service_longevity_with_limits(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86_400.0,
                &events(),
                battery,
            )
            .unwrap();
            let canonical_days = canonical(&model);
            let legacy_days = legacy.survived_s / 86_400.0;
            let class =
                if legacy_days.round() == canonical_days.round() { "EXPECTED_AGREEMENT" } else { "UNEXPLAINED" };
            println!(
                "joint-yield\tyield={fraction:.1},bronze={bronze:.1}\t{legacy_days:.0}\t{canonical_days:.3}\t{class}"
            );
        }
    }
    ExitCode::SUCCESS
}
