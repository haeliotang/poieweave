// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 mass-conserving recycling with residue: how much bar recovery is needed?

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{Flow, Model};
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let base = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    let mut events: Vec<FailureEvent> =
        (1..=8).map(|index| FailureEvent { at_s: f64::from(index * 10) * 86400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86400.0, asset: "lathe-1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    println!("bar_recovery_kg\tresidue_kg\tsurvived_days\tblocked_by");
    for (bar_kg, residue_kg) in [(1.2, 0.0), (1.0, 0.2), (0.9, 0.3), (0.6, 0.6)] {
        let mut model = base.clone();
        let process = model.processes.iter_mut().find(|process| process.id == "recycle-bronze").unwrap();
        for output in &mut process.outputs {
            if output.resource == "bronze-bar" {
                output.si = bar_kg;
            }
        }
        if residue_kg > 0.0 {
            process.outputs.push(Flow { resource: "bronze-slag".into(), si: residue_kg });
        }
        let report = simulate_multi_service_longevity_with_limits(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86400.0,
            &events,
            battery,
        )
        .unwrap();
        let blocked = report.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
        println!("{bar_kg:.1}\t{residue_kg:.1}\t{:.0}\t{blocked}", report.survived_s / 86400.0);
    }
    ExitCode::SUCCESS
}
