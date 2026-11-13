// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 survival-horizon curve: pump-failure interval versus modelled survival.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_multi_service_longevity_with_limits, BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-05/model.toml")).unwrap_or_else(|error| {
        panic!("load Base-05: {error}");
    });
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("interval_days\tsurvived_days\tfailed_at\tfinal_energy_kwh\tblocked_by");
    for interval in [10_u32, 15, 20, 30] {
        let mut events: Vec<FailureEvent> = (1..=(89 / interval))
            .map(|index| FailureEvent { at_s: f64::from(index * interval) * 86400.0, asset: "pump-p1".into() })
            .collect();
        events.push(FailureEvent { at_s: 35.0 * 86400.0, asset: "lathe-1".into() });
        events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
        let report = simulate_multi_service_longevity_with_limits(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86400.0,
            &events,
            battery,
        )
        .unwrap();
        let blocked = report.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
        println!(
            "{interval}\t{:.0}\tday {:.0}\t{:.1}\t{blocked}",
            report.survived_s / 86400.0,
            report.survived_s / 86400.0,
            report.final_inventory.get("electric-energy").copied().unwrap_or(0.0) / 3_600_000.0
        );
    }
    ExitCode::SUCCESS
}
