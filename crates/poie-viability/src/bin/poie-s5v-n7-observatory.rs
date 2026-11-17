// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later
//! N7 native asset-state observation evidence runner.

use std::path::{Path, PathBuf};

use poie_ontology::model::{AssetStatus, Model};
use poie_viability::engine::{
    run_engine, run_engine_observed, EngineOptions, EngineTermination, ObservationPhase, ObservationRequest,
};
use poie_viability::survival::{BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn label(term: &EngineTermination) -> String {
    match term {
        EngineTermination::CompletedHorizon => "completed".into(),
        EngineTermination::Blocked(reason) => format!("blocked:{reason}"),
        EngineTermination::IncompleteAtHorizon(reason) => format!("incomplete:{reason}"),
    }
}
fn options(failures: Vec<FailureEvent>) -> EngineOptions<'static> {
    EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s: 20.0 * 3600.0,
        failures,
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        load_kw: 3.0,
    }
}
fn main() {
    let out = root().join("audit/phase7");
    std::fs::create_dir_all(out.join("observations")).unwrap();
    let cases = [
        (
            "N7-00",
            Vec::new(),
            vec![
                (0.0, ObservationPhase::After),
                (3600.0, ObservationPhase::After),
                (7200.0, ObservationPhase::After),
                (10800.0, ObservationPhase::Before),
                (10800.0, ObservationPhase::After),
                (14400.0, ObservationPhase::After),
            ],
        ),
        (
            "N7-03",
            vec![FailureEvent { at_s: 7200.0, asset: "robot-m1".into() }],
            vec![
                (7200.0, ObservationPhase::Before),
                (7200.0, ObservationPhase::After),
                (20000.0, ObservationPhase::After),
            ],
        ),
    ];
    let mut rows =
        String::from("case\tat_s\tphase\tstatus\tasset_id\tasset_status\tstatus_source\trunning_repair\ttermination\n");
    let mut summaries = Vec::new();
    for (case, failures, samples) in cases {
        let model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
        let opts = options(failures);
        let requests: Vec<ObservationRequest> =
            samples.into_iter().map(|(at_s, phase)| ObservationRequest { at_s, phase }).collect();
        let direct = run_engine(&model, &opts).unwrap();
        let observed = run_engine_observed(&model, &opts, &requests).unwrap();
        assert_eq!(direct, observed.outcome, "noninterference {case}");
        for observation in &observed.observations {
            if observation.asset_statuses.is_empty() {
                rows.push_str(&format!(
                    "{case}\t{:.3}\t{:?}\t{}\t*\tUNOBSERVABLE\t{}\t{}\t{}\n",
                    observation.at_s,
                    observation.phase,
                    observation.status.label(),
                    observation.asset_status_source,
                    observation.running_repair.as_ref().map(|r| r.label.as_str()).unwrap_or("NONE"),
                    label(&observed.outcome.termination)
                ));
            } else {
                for (asset, status) in &observation.asset_statuses {
                    let name = match status {
                        AssetStatus::Available => "Available",
                        AssetStatus::Degraded => "Degraded",
                        AssetStatus::Failed => "Failed",
                        AssetStatus::Unknown => "Unknown",
                    };
                    rows.push_str(&format!(
                        "{case}\t{:.3}\t{:?}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                        observation.at_s,
                        observation.phase,
                        observation.status.label(),
                        asset,
                        name,
                        observation.asset_status_source,
                        observation.running_repair.as_ref().map(|r| r.label.as_str()).unwrap_or("NONE"),
                        label(&observed.outcome.termination)
                    ));
                }
            }
        }
        summaries.push(format!(
            "{{\"case\":\"{case}\",\"termination\":\"{}\",\"survived_s\":{:.3},\"noninterference\":true}}",
            label(&observed.outcome.termination),
            observed.outcome.survived_s
        ));
    }
    std::fs::write(out.join("observations/asset-statuses.csv"), rows).unwrap();
    std::fs::write(
        out.join("observations/summary.json"),
        format!("{{\"schema\":\"poie.s5v-n7-observations/1\",\"cases\":[{}]}}\n", summaries.join(",")),
    )
    .unwrap();
    println!("wrote N7 observations");
}
