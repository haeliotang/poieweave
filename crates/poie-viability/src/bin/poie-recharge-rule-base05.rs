// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2: does the load switch track average PV recharge when recharge changes?

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::self_sustain::RechargeBoundary;
use poie_viability::survival::{simulate_multi_service_longevity_with_load, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn with_recharge(mut model: Model, kwh: f64) -> Model {
    let process = model.processes.iter_mut().find(|process| process.id == "recharge-energy").unwrap();
    for output in &mut process.outputs {
        if output.resource == "electric-energy" {
            output.si = kwh * 3_600_000.0;
        }
    }
    model
}

fn first_energy_limited(model: &Model, max_load_tenths: u32) -> f64 {
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86400.0, asset: "lathe-1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    for tenths in 1..=max_load_tenths {
        let load_kw = f64::from(tenths) / 10.0;
        let report = simulate_multi_service_longevity_with_load(
            model,
            &["potable-water", "workshop-operability"],
            90.0 * 86400.0,
            &events,
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
            load_kw,
        )
        .unwrap();
        if report.log.iter().any(|entry| entry.blocked_by.as_deref() == Some("energy_depleted")) {
            return load_kw;
        }
    }
    0.0
}

fn main() -> ExitCode {
    let root = root();
    let base = Model::load(&root.join("scenarios/base-05/model.toml")).unwrap();
    println!("recharge_kwh\taverage_kw\tfirst_energy_limited_load_kw\tmargin_kw");
    for kwh in [10.0, 20.0, 40.0] {
        let average_kw = kwh / 8.0;
        let model = with_recharge(base.clone(), kwh);
        let threshold = first_energy_limited(&model, 80);
        let boundary = RechargeBoundary { average_recharge_kw: average_kw, threshold_load_kw: threshold };
        println!("{kwh:.0}\t{average_kw:.2}\t{threshold:.1}\t{:.2}", boundary.margin_kw());
    }
    ExitCode::SUCCESS
}
