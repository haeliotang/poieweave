// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later
//! N7 native asset-state observation evidence runner.

use std::path::{Path, PathBuf};

use poie_ontology::model::{AssetStatus, Model, Service};
use poie_viability::closure::{analyze, ClosureOutcome, Plan};
use poie_viability::engine::{
    run_engine, run_engine_observed, EngineOptions, EngineTermination, ObservationPhase, ObservationRequest,
};
use poie_viability::scheduler::schedule_plans_with_alternatives;
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
fn options(failures: Vec<FailureEvent>, horizon_s: f64) -> EngineOptions<'static> {
    EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s,
        failures,
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        load_kw: 3.0,
    }
}
fn n6_synthetic() -> Model {
    let mut model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    model.assets.get_mut("lathe-1").unwrap().status = AssetStatus::Failed;
    let robot = model.assets.get("robot-m1").unwrap().clone();
    let mut second = robot.clone();
    second.id = "robot-m2".into();
    model.assets.insert("robot-m2".into(), second);
    for process in &mut model.processes {
        if process.id == "replace-sleeve-p1" || process.id == "replace-lathe-nut" {
            process.occupies = vec!["robot-m1".into()];
        }
    }
    for capability in ["sleeve-replacement", "lathe-repair", "machine-tending", "material-handling"] {
        model.providers.entry(capability.into()).or_default().push("robot-m2".into());
    }
    model
}

fn tool_model() -> Model {
    let mut model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let robot = model.assets.get("robot-m1").unwrap().clone();
    let mut tool = robot.clone();
    tool.id = "tool-t1".into();
    tool.kind = "tool".into();
    tool.status = AssetStatus::Available;
    tool.requires.clear();
    tool.requires_any.clear();
    tool.provides.clear();
    model.assets.insert("tool-t1".into(), tool);
    let pump = model.assets.get("pump-p1").unwrap().clone();
    let template = model.processes.iter().find(|process| process.id == "replace-sleeve-p1").unwrap().clone();
    let mut asset = pump.clone();
    asset.id = "a-lathe".into();
    asset.status = AssetStatus::Failed;
    asset.requires.clear();
    asset.requires_any.clear();
    asset.provides.clear();
    model.assets.insert("a-lathe".into(), asset);
    let mut process = template;
    process.id = "repair-a-lathe".into();
    process.inputs.clear();
    process.outputs.clear();
    process.requires_capabilities.clear();
    process.occupies = vec!["robot-m1".into(), "tool-t1".into()];
    process.restores = vec!["a-lathe".into()];
    process.duration_s = Some(10800.0);
    let index = model.processes.len();
    model.processes.push(process);
    model.restorers.entry("a-lathe".into()).or_default().push(index);
    model.services.insert(
        "n7-tool".into(),
        Service { id: "n7-tool".into(), requires: vec!["a-lathe".into()], requires_any: Vec::new() },
    );
    model
}

fn plan_for(model: &Model, target: &str, horizon_s: f64) -> Plan {
    match analyze(model, target, horizon_s) {
        ClosureOutcome::Restorable(plan) => plan,
        ClosureOutcome::Blocked(blocked) => panic!("analyze {target} blocked: {:?}", blocked.reasons),
    }
}

fn main() {
    let out = root().join("audit/phase7");
    std::fs::create_dir_all(out.join("observations")).unwrap();
    let cases = [
        (
            "N7-00",
            Vec::new(),
            20.0 * 3600.0,
            false,
            vec![
                (0.0, ObservationPhase::After),
                (3600.0, ObservationPhase::After),
                (7200.0, ObservationPhase::After),
                (10800.0, ObservationPhase::Before),
                (10800.0, ObservationPhase::After),
                (14400.0, ObservationPhase::After),
            ],
        ),
        ("N7-01", Vec::new(), 20.0 * 3600.0, false, vec![(0.0, ObservationPhase::After)]),
        (
            "N7-02",
            Vec::new(),
            20.0 * 3600.0,
            false,
            vec![(10800.0, ObservationPhase::Before), (10800.0, ObservationPhase::After)],
        ),
        (
            "N7-03",
            vec![FailureEvent { at_s: 7200.0, asset: "robot-m1".into() }],
            20.0 * 3600.0,
            false,
            vec![
                (7200.0, ObservationPhase::Before),
                (7200.0, ObservationPhase::After),
                (20000.0, ObservationPhase::After),
            ],
        ),
        (
            "N7-04",
            vec![FailureEvent { at_s: 3600.0, asset: "lathe-1".into() }],
            20.0 * 3600.0,
            false,
            vec![
                (3600.0, ObservationPhase::Before),
                (3600.0, ObservationPhase::After),
                (10800.0, ObservationPhase::After),
            ],
        ),
        (
            "N7-06",
            vec![FailureEvent { at_s: 3600.0, asset: "bus-1".into() }],
            20.0 * 3600.0,
            false,
            vec![(3600.0, ObservationPhase::Before), (3600.0, ObservationPhase::After)],
        ),
        (
            "N7-07",
            vec![FailureEvent { at_s: 10800.0, asset: "robot-m1".into() }],
            20.0 * 3600.0,
            false,
            vec![
                (10800.0, ObservationPhase::Before),
                (10800.0, ObservationPhase::After),
                (10800.0, ObservationPhase::After),
                (10800.0, ObservationPhase::After),
                (10800.001, ObservationPhase::After),
            ],
        ),
        (
            "N7-08",
            Vec::new(),
            10800.0,
            false,
            vec![(10800.0, ObservationPhase::Before), (10800.0, ObservationPhase::After)],
        ),
        (
            "N7-09",
            vec![FailureEvent { at_s: 10800.0, asset: "robot-m1".into() }],
            10800.0,
            false,
            vec![(10800.0, ObservationPhase::Before), (10800.0, ObservationPhase::After)],
        ),
        (
            "N7-10",
            vec![FailureEvent { at_s: 7200.0, asset: "robot-m1".into() }],
            20.0 * 3600.0,
            false,
            vec![(7200.0, ObservationPhase::After), (20000.0, ObservationPhase::After)],
        ),
        (
            "N7-11",
            vec![
                FailureEvent { at_s: 3600.0, asset: "robot-m1".into() },
                FailureEvent { at_s: 7200.0, asset: "lathe-1".into() },
            ],
            20.0 * 3600.0,
            false,
            vec![(3600.0, ObservationPhase::After), (7200.0, ObservationPhase::After)],
        ),
        ("N7-12", Vec::new(), 20.0 * 3600.0, true, vec![(0.0, ObservationPhase::After)]),
        (
            "N7-13",
            Vec::new(),
            20.0 * 3600.0,
            false,
            vec![
                (3600.0, ObservationPhase::After),
                (3600.0, ObservationPhase::After),
                (10800.0, ObservationPhase::Before),
                (10800.0, ObservationPhase::After),
            ],
        ),
    ];
    let mut rows =
        String::from("case\tat_s\tphase\tstatus\tasset_id\tasset_status\tstatus_source\trunning_repair\ttermination\n");
    let mut summaries = Vec::new();
    for (case, failures, horizon, unknown_asset, samples) in cases {
        let mut model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
        if unknown_asset {
            model.assets.get_mut("lathe-1").unwrap().status = AssetStatus::Unknown;
        }
        let opts = options(failures, horizon);
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
    // N7-05: required tool failure observed natively.
    {
        let model = tool_model();
        let horizon = 20.0 * 3600.0;
        let opts = EngineOptions {
            services: vec!["n7-tool"],
            horizon_s: horizon,
            failures: vec![FailureEvent { at_s: 9000.0, asset: "tool-t1".into() }],
            resupplies: Vec::new(),
            resources: vec!["robot-m1".into()],
            alternative_pool: vec!["robot-m1".into()],
            battery: None,
            load_kw: 0.0,
        };
        let requests = [
            ObservationRequest { at_s: 9000.0, phase: ObservationPhase::Before },
            ObservationRequest { at_s: 9000.0, phase: ObservationPhase::After },
        ];
        let direct = run_engine(&model, &opts).unwrap();
        let observed = run_engine_observed(&model, &opts, &requests).unwrap();
        assert_eq!(direct, observed.outcome, "noninterference N7-05");
        for observation in &observed.observations {
            for (asset, status) in &observation.asset_statuses {
                let name = match status {
                    AssetStatus::Available => "Available",
                    AssetStatus::Degraded => "Degraded",
                    AssetStatus::Failed => "Failed",
                    AssetStatus::Unknown => "Unknown",
                };
                rows.push_str(&format!(
                    "N7-05\t{:.3}\t{:?}\t{}\t{}\t{}\t{}\t{}\t{}\n",
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
        summaries.push(format!(
            "{{\"case\":\"N7-05\",\"termination\":\"{}\",\"survived_s\":{:.3},\"noninterference\":true}}",
            label(&observed.outcome.termination),
            observed.outcome.survived_s
        ));
    }
    // N7-14: overlap with N6-A-10800 derived restoration evidence.
    {
        let model = n6_synthetic();
        let horizon = 20.0 * 3600.0;
        let targets = ["lathe-1", "pump-p1"];
        let plans = targets.iter().map(|target| plan_for(&model, target, horizon)).collect::<Vec<_>>();
        let pool = vec!["robot-m1".to_string(), "robot-m2".to_string()];
        let schedule =
            schedule_plans_with_alternatives(&model, &plans, &pool, &pool, std::slice::from_ref(&pool)).unwrap();
        let opts = EngineOptions {
            services: vec!["potable-water", "workshop-operability"],
            horizon_s: horizon,
            failures: vec![FailureEvent { at_s: 10800.0, asset: "robot-m1".into() }],
            resupplies: Vec::new(),
            resources: pool.clone(),
            alternative_pool: pool.clone(),
            battery: None,
            load_kw: 0.0,
        };
        let requests = [
            ObservationRequest { at_s: schedule.makespan_s, phase: ObservationPhase::Before },
            ObservationRequest { at_s: schedule.makespan_s, phase: ObservationPhase::After },
        ];
        let direct = run_engine(&model, &opts).unwrap();
        let observed = run_engine_observed(&model, &opts, &requests).unwrap();
        assert_eq!(direct, observed.outcome, "noninterference N7-14");
        for observation in &observed.observations {
            for (asset, status) in &observation.asset_statuses {
                let name = match status {
                    AssetStatus::Available => "Available",
                    AssetStatus::Degraded => "Degraded",
                    AssetStatus::Failed => "Failed",
                    AssetStatus::Unknown => "Unknown",
                };
                rows.push_str(&format!(
                    "N7-14\t{:.3}\t{:?}\t{}\t{}\t{}\t{}\t{}\t{}\n",
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
        summaries.push(format!(
            "{{\"case\":\"N7-14\",\"termination\":\"{}\",\"survived_s\":{:.3},\"noninterference\":true}}",
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
