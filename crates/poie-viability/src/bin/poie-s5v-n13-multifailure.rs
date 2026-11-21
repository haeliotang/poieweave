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
    let out = root().join("audit/phase13/canonical");
    std::fs::create_dir_all(&out).unwrap();
    let cases: [(&str, usize, bool, bool, Vec<FailureEvent>); 10] = [
        ("N13-00", 1, false, false, Vec::new()),
        ("N13-01", 1, false, false, vec![FailureEvent { at_s: 3600.0, asset: "robot-m1".into() }]),
        (
            "N13-02",
            1,
            false,
            true,
            vec![
                FailureEvent { at_s: 3600.0, asset: "sensor-x".into() },
                FailureEvent { at_s: 5400.0, asset: "bus-2".into() },
            ],
        ),
        (
            "N13-03",
            1,
            false,
            true,
            vec![
                FailureEvent { at_s: 3600.0, asset: "bus-2".into() },
                FailureEvent { at_s: 5400.0, asset: "sensor-x".into() },
            ],
        ),
        (
            "N13-04",
            1,
            false,
            true,
            vec![
                FailureEvent { at_s: 3600.0, asset: "sensor-x".into() },
                FailureEvent { at_s: 3600.0, asset: "bus-2".into() },
            ],
        ),
        (
            "N13-05",
            1,
            false,
            true,
            vec![
                FailureEvent { at_s: 3600.0, asset: "sensor-x".into() },
                FailureEvent { at_s: 5400.0, asset: "robot-m1".into() },
            ],
        ),
        (
            "N13-06",
            1,
            false,
            true,
            vec![
                FailureEvent { at_s: 3600.0, asset: "robot-m1".into() },
                FailureEvent { at_s: 5400.0, asset: "sensor-x".into() },
            ],
        ),
        (
            "N13-07",
            1,
            false,
            true,
            vec![
                FailureEvent { at_s: 3600.0, asset: "bus-1".into() },
                FailureEvent { at_s: 5400.0, asset: "sensor-x".into() },
            ],
        ),
        (
            "N13-08",
            1,
            false,
            true,
            vec![
                FailureEvent { at_s: 3600.0, asset: "bus-2".into() },
                FailureEvent { at_s: 5400.0, asset: "bus-1".into() },
            ],
        ),
        (
            "N13-09",
            2,
            false,
            true,
            vec![
                FailureEvent { at_s: 3600.0, asset: "robot-m1".into() },
                FailureEvent { at_s: 3600.0, asset: "robot-m2".into() },
            ],
        ),
    ];
    for (case, robots, tool, capability, failures) in cases {
        let model = model_for(robots, tool, capability);
        let pool = if robots > 1 { vec!["robot-m1".into(), "robot-m2".into()] } else { vec!["robot-m1".into()] };
        let options = EngineOptions {
            services: vec!["n10-service"],
            horizon_s: 72000.0,
            failures,
            resupplies: Vec::new(),
            resources: pool.clone(),
            alternative_pool: pool,
            battery: None,
            load_kw: 0.0,
        };
        let direct = run_engine(&model, &options).unwrap();
        let observed = run_engine_event_observed(&model, &options).unwrap();
        assert_eq!(direct, observed.outcome, "noninterference {case}");
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
            let inventory_before: serde_json::Map<String, serde_json::Value> =
                event.inventory_before.iter().map(|(k, v)| (k.clone(), serde_json::json!(v))).collect();
            let inventory_after: serde_json::Map<String, serde_json::Value> =
                event.inventory_after.iter().map(|(k, v)| (k.clone(), serde_json::json!(v))).collect();
            let repair = event.running_repair.as_ref().map(|r| serde_json::json!({
                "label": r.label, "end_s": r.end_s, "plans": r.plans, "resources": r.resources,
                "execution_dependencies": r.execution_dependencies, "restoration_targets": r.restoration_targets,
                "task_resources": r.task_resources.iter().map(|e| serde_json::json!({"plan_index": e.plan_index, "resources": e.resources})).collect::<Vec<_>>(),
            }));
            let value = serde_json::json!({
                "schema": "poie.s5v-n12-canonical-event/1", "case": case,
                "event_id": event.event_id, "event_sequence": event.event_sequence,
                "event_batch_sequence": event.event_batch_sequence, "at_s": event.at_s,
                "event_kind": event.kind.label(), "event_origin": format!("{:?}", event.origin),
                "target_asset": event.target_asset, "event_status": format!("{:?}", event.status),
                "asset_statuses_before": before, "asset_statuses_after": after,
                "inventory_before": inventory_before, "inventory_after": inventory_after,
                "running_repair": repair, "observation_source": "NATIVE_CANONICAL_RUNTIME",
            });
            lines.push_str(&serde_json::to_string(&value).unwrap());
            lines.push('\n');
        }
        std::fs::write(out.join(format!("{case}-events.jsonl")), lines).unwrap();
        let repair = observed.events.first().and_then(|event| event.running_repair.as_ref());
        let resources = repair.map(|r| r.resources.clone()).unwrap_or_default();
        let tasks = repair
            .map(|r| {
                r.task_resources
                    .iter()
                    .map(|e| format!("{}:{}", e.plan_index, e.resources.join("+")))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let witness = repair
            .map(|r| {
                r.witness
                    .iter()
                    .map(|w| format!("{}:{}:{}", w.capability, w.provider, w.dependencies.join("|")))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let completion = observed
            .events
            .iter()
            .find(|e| e.kind == poie_viability::engine::EventKind::RepairCompleted)
            .map(|e| e.at_s);
        let interruption = observed
            .events
            .iter()
            .find(|e| e.kind == poie_viability::engine::EventKind::RepairInterrupted)
            .map(|e| e.at_s);
        let summary = format!("{{\n \"case\":\"{case}\",\n \"resources\":[{}],\n \"task_resources\":[{}],\n \"witness\":[{}],\n \"completion_s\":{},\n \"interruption_s\":{},\n \"restoration_committed\":{},\n \"termination\":\"{}\"\n}}\n",
            resources.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(","),
            tasks.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(","),
            witness.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(","),
            completion.map(|v| format!("{v:.3}")).unwrap_or_else(|| "null".into()),
            interruption.map(|v| format!("{v:.3}")).unwrap_or_else(|| "null".into()),
            completion.is_some(), label(&observed.outcome.termination));
        std::fs::write(out.join(format!("{case}-summary.json")), summary).unwrap();
        println!("{case}: completion={completion:?} interruption={interruption:?} resources={resources:?}");
    }
}
