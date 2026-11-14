// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 joint consumable frontier under site load.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{Flow, Model, Process, Verification};
use poie_viability::survival::{simulate_multi_service_longevity_with_load, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sharpen_process() -> Process {
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
        process_yield: None,
    }
}

fn main() -> ExitCode {
    let root = root();
    let base = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86400.0, asset: "lathe-1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    println!("load_kw\tbronze_kg\tsurvived_days\tblocked_by");
    for load_kw in [2.0, 3.0] {
        for bronze in [3.0, 6.0] {
            let mut model = base.clone();
            model.add_process(sharpen_process());
            if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "cutting-insert") {
                lot.si = 2.0;
            }
            if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "bronze-bar") {
                lot.si = bronze;
            }
            let report = simulate_multi_service_longevity_with_load(
                &model,
                &["potable-water", "workshop-operability"],
                90.0 * 86400.0,
                &events,
                battery,
                load_kw,
            )
            .unwrap();
            let blocked = report.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
            println!("{load_kw:.0}\t{bronze:.1}\t{:.0}\t{blocked}", report.survived_s / 86400.0);
        }
    }
    ExitCode::SUCCESS
}
