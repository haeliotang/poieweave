// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2: insert boundary under instantaneous vs clock-advanced repair time.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{
    simulate_multi_service_longevity_event_driven, simulate_multi_service_longevity_with_load, BatteryLimits,
    FailureEvent,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let base = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let events: Vec<FailureEvent> =
        (1..=15).map(|index| FailureEvent { at_s: f64::from(index * 2) * 86_400.0, asset: "pump-p1".into() }).collect();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("inserts\tinstant_days\tevent_driven_days");
    for inserts in [1.0, 2.0, 3.0, 4.0, 6.0] {
        let mut model = base.clone();
        model.lots.iter_mut().find(|lot| lot.id == "lot-insert").unwrap().si = inserts;
        model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap().duration_s =
            Some(5.0 * 86_400.0);
        let old = simulate_multi_service_longevity_with_load(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &events,
            battery,
            0.0,
        )
        .unwrap();
        let new = simulate_multi_service_longevity_event_driven(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &events,
            battery,
            0.0,
        )
        .unwrap();
        println!("{inserts:.0}\t{:.0}\t{:.0}", old.survived_s / 86400.0, new.survived_s / 86400.0);
    }
    ExitCode::SUCCESS
}
