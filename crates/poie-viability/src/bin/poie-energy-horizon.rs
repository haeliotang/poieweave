// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S4: export no-event energy horizons for an independent model.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_multi_service_longevity_with_load, BatteryLimits};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    println!("load_kw\tsurvived_days\tblocked_by");
    for load_kw in [2.5, 2.6, 3.0, 4.0] {
        let report = simulate_multi_service_longevity_with_load(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86400.0,
            &[],
            battery,
            load_kw,
        )
        .unwrap();
        let blocked = report.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
        println!("{load_kw:.1}\t{:.0}\t{blocked}", report.survived_s / 86400.0);
    }
    ExitCode::SUCCESS
}
