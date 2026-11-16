// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase B: C14 redundancy and repair time under dense failures.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{Asset, AssetStatus, Model};
use poie_viability::engine::{run_engine, EngineOptions, EngineTermination};
use poie_viability::survival::{
    simulate_multi_service_longevity_event_driven, simulate_multi_service_longevity_with_load, BatteryLimits,
    FailureEvent,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let base = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
    let mut events: Vec<FailureEvent> =
        (1..=45).map(|index| FailureEvent { at_s: f64::from(index * 2) * 86_400.0, asset: "pump-p1".into() }).collect();
    events.push(FailureEvent { at_s: 35.0 * 86_400.0, asset: "lathe-1".into() });
    events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    println!("mode\tload_kw\tinstant_days\tevent_days\tcanonical_days\tevent_energy_kwh\tcanonical_energy_kwh\tcanonical_termination\tclassification");
    for mode in ["baseline", "redundant-pump"] {
        let load_kw = 0.0;
        let mut model = base.clone();
        model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap().duration_s =
            Some(5.0 * 86_400.0);
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
        let instant = simulate_multi_service_longevity_with_load(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &events,
            battery,
            load_kw,
        )
        .unwrap();
        let event = simulate_multi_service_longevity_event_driven(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86_400.0,
            &events,
            battery,
            load_kw,
        )
        .unwrap();
        let options = EngineOptions {
            services: vec!["potable-water", "workshop-operability"],
            horizon_s: 90.0 * 86_400.0,
            failures: events.clone(),
            resupplies: Vec::new(),
            resources: vec!["robot-m1".into(), "lathe-1".into()],
            alternative_pool: vec!["robot-m1".into()],
            battery,
            load_kw,
        };
        let canonical = run_engine(&model, &options).unwrap();
        let event_days = event.survived_s / 86_400.0;
        let canonical_days = canonical.survived_s / 86_400.0;
        let event_energy = event.final_inventory["electric-energy"] / 3_600_000.0;
        let canonical_energy = canonical.final_inventory["electric-energy"] / 3_600_000.0;
        let termination = match &canonical.termination {
            EngineTermination::CompletedHorizon => "completed".to_string(),
            EngineTermination::Blocked(reason) => format!("blocked:{reason}"),
            EngineTermination::IncompleteAtHorizon(reason) => format!("incomplete:{reason}"),
        };
        let classification = if (event_days - canonical_days).abs() < 1e-9
            && (event_energy - canonical_energy).abs() < 1e-6
        {
            "EXPECTED_AGREEMENT"
        } else if matches!(&canonical.termination, EngineTermination::Blocked(reason) if reason.contains("interrupted"))
        {
            "SEMANTIC_CORRECTION_IN_REPAIR_FAILURE"
        } else {
            "UNEXPLAINED"
        };
        println!(
            "{mode}\t{load_kw:.1}\t{:.0}\t{event_days:.0}\t{canonical_days:.0}\t{event_energy:.1}\t{canonical_energy:.1}\t{termination}\t{classification}",
            instant.survived_s / 86_400.0
        );
    }
    ExitCode::SUCCESS
}
