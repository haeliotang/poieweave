// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 redundancy vs a shared repair robot failure.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{Asset, AssetStatus, Model};
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
    events.push(FailureEvent { at_s: 20.0 * 86400.0, asset: "robot-m1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    println!("mode\tsurvived_days\tblocked_by");
    for mode in ["baseline", "redundant-pump"] {
        let mut model = base.clone();
        if mode == "redundant-pump" {
            model.assets.insert(
                "pump-p2".into(),
                Asset {
                    id: "pump-p2".into(),
                    kind: "pump".into(),
                    status: AssetStatus::Available,
                    status_evidence: None,
                    requires: Vec::new(),
                    requires_any: Vec::new(),
                    provides: Vec::new(),
                    capability_verification: BTreeMap::new(),
                    degraded: None,
                },
            );
            let service = model.services.get_mut("potable-water").unwrap();
            service.requires.retain(|asset| asset != "pump-p1");
            service.requires_any.push("pump-p1".into());
            service.requires_any.push("pump-p2".into());
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
        println!("{mode}\t{:.0}\t{blocked}", report.survived_s / 86400.0);
    }
    ExitCode::SUCCESS
}
