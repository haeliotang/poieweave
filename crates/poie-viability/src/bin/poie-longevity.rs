// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 long-horizon runner: repeated pump failures on Base-05 under a greedy
//! repair policy with PV recharge and shared inventory.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_longevity, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-05/model.toml")).unwrap_or_else(|error| {
        panic!("load Base-05: {error}");
    });
    let horizon = 90.0 * 86400.0;
    let events: Vec<FailureEvent> = (1..=9)
        .map(|index| FailureEvent { at_s: f64::from(index) * 10.0 * 86400.0, asset: "pump-p1".into() })
        .collect();
    let report = simulate_longevity(&model, "potable-water", horizon, &events).unwrap();
    println!(
        "service={} horizon_days=90 survived_days={:.0} final_energy_kwh={:.1}",
        report.service,
        report.survived_s / 86400.0,
        report.final_inventory.get("electric-energy").copied().unwrap_or(0.0) / 3_600_000.0
    );
    for entry in &report.log {
        let failed = entry.failed.as_deref().unwrap_or("none");
        let repaired = if entry.repaired.is_empty() { "none".to_string() } else { entry.repaired.join(",") };
        let status = entry.blocked_by.as_deref().unwrap_or("ok");
        println!(
            "event\tday={:.0}\tfailed={failed}\trepaired={repaired}\tstatus={status}\tenergy_kwh={:.1}",
            entry.at_s / 86400.0,
            entry.energy_after / 3_600_000.0
        );
    }
    ExitCode::SUCCESS
}
