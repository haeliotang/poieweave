// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later
//! N10 named-upstream and alternative-resource canonical evidence runner.

use poie_ontology::model::{AssetStatus, Model, Service};
use poie_viability::engine::{run_engine, run_engine_event_observed, EngineOptions, EngineTermination};
use poie_viability::survival::FailureEvent;
use std::path::{Path, PathBuf};

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

fn model_for(robots: usize, tool: bool, capability: bool) -> Model {
    let mut model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let robot = model.assets.get("robot-m1").unwrap().clone();
    if robots > 1 {
        let mut second = robot.clone();
        second.id = "robot-m2".into();
        model.assets.insert("robot-m2".into(), second);
    }
    if tool {
        let mut tool_asset = robot.clone();
        tool_asset.id = "tool-t1".into();
        tool_asset.kind = "tool".into();
        tool_asset.requires.clear();
        tool_asset.requires_any.clear();
        tool_asset.provides.clear();
        model.assets.insert("tool-t1".into(), tool_asset);
    }
    for extra in ["sensor-x", "bus-2"] {
        let mut asset = robot.clone();
        asset.id = extra.into();
        asset.kind = "sensor".into();
        asset.requires.clear();
        asset.requires_any.clear();
        asset.provides.clear();
        model.assets.insert(extra.into(), asset);
    }
    let pump = model.assets.get("pump-p1").unwrap().clone();
    let mut asset = pump;
    asset.id = "a-lathe".into();
    asset.status = AssetStatus::Failed;
    asset.requires.clear();
    asset.requires_any.clear();
    asset.provides.clear();
    model.assets.insert("a-lathe".into(), asset);
    let template = model.processes.iter().find(|process| process.id == "replace-sleeve-p1").unwrap().clone();
    let mut process = template;
    process.id = "n10-repair-a-lathe".into();
    process.inputs.clear();
    process.outputs.clear();
    process.requires_capabilities.clear();
    process.occupies = vec!["robot-m1".into()];
    if tool {
        process.occupies.push("tool-t1".into());
    }
    if capability {
        process.requires_capabilities = vec!["sleeve-replacement".into()];
    }
    process.restores = vec!["a-lathe".into()];
    process.duration_s = Some(7200.0);
    let index = model.processes.len();
    model.processes.push(process);
    model.restorers.insert("a-lathe".into(), vec![index]);
    if robots > 1 {
        for provider in ["robot-m1", "robot-m2"] {
            for capability in ["sleeve-replacement", "lathe-repair", "machine-tending", "material-handling"] {
                model.providers.entry(capability.into()).or_default().push(provider.into());
            }
        }
    }
    model.services.insert(
        "n10-service".into(),
        Service { id: "n10-service".into(), requires: vec!["a-lathe".into()], requires_any: Vec::new() },
    );
    model
}

fn main() {
    let path = std::env::args().nth(1).expect("contract json path");
    let text = std::fs::read_to_string(&path).unwrap();
    let c: serde_json::Value = serde_json::from_str(&text).unwrap();
    let tasks = c["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 1, "N14 canonical adapter supports exactly one task");
    let task = &tasks[0];
    let duration = task["duration_s"].as_f64().unwrap();
    let robots = c["robot_count"].as_u64().unwrap_or(1) as usize;
    let tool = task["requires_tool"].as_bool().unwrap_or(false);
    let mut model = model_for(robots, tool, true);
    if let Some(process) = model.processes.iter_mut().find(|p| p.id == "n10-repair-a-lathe") {
        process.duration_s = Some(duration);
    }
    let failures: Vec<FailureEvent> = c["failures"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|f| FailureEvent {
                    at_s: f["time_s"].as_f64().unwrap(),
                    asset: f["target"].as_str().unwrap().to_string(),
                })
                .collect()
        })
        .unwrap_or_default();
    let horizon = c["horizon_s"].as_f64().unwrap();
    let options = EngineOptions {
        services: vec!["n10-service"],
        horizon_s: horizon,
        failures,
        resupplies: Vec::new(),
        resources: if robots > 1 { vec!["robot-m1".into(), "robot-m2".into()] } else { vec!["robot-m1".into()] },
        alternative_pool: if robots > 1 { vec!["robot-m1".into(), "robot-m2".into()] } else { vec!["robot-m1".into()] },
        battery: None,
        load_kw: 0.0,
    };
    let observed = run_engine_event_observed(&model, &options).unwrap();
    let mut lines = String::new();
    for event in &observed.events {
        let before: serde_json::Map<String, serde_json::Value> = event
            .asset_statuses_before
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(format!("{v:?}"))))
            .collect();
        let after: serde_json::Map<String, serde_json::Value> = event
            .asset_statuses_after
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(format!("{v:?}"))))
            .collect();
        let repair = event.running_repair.as_ref().map(|r| serde_json::json!({"resources": r.resources, "task_resources": r.task_resources.iter().map(|e| serde_json::json!({"plan_index": e.plan_index, "resources": e.resources})).collect::<Vec<_>>(), "execution_dependencies": r.execution_dependencies, "restoration_targets": r.restoration_targets}));
        let v = serde_json::json!({"schema":"poie.s5v-n14-canonical-event/1","event_sequence":event.event_sequence,"at_s":event.at_s,"event_kind":event.kind.label(),"target_asset":event.target_asset,"asset_statuses_before":before,"asset_statuses_after":after,"running_repair":repair,"observation_source":"NATIVE_CANONICAL_RUNTIME"});
        lines.push_str(&serde_json::to_string(&v).unwrap());
        lines.push('\n');
    }
    print!("{lines}");
}
