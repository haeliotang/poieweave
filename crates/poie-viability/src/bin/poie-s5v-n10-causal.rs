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
    let out = root().join("audit/phase10/canonical");
    std::fs::create_dir_all(&out).unwrap();
    let cases: [(&str, usize, bool, bool, Vec<FailureEvent>); 7] = [
        ("N10-U01", 1, false, true, vec![FailureEvent { at_s: 3600.0, asset: "bus-1".into() }]),
        ("N10-U02", 1, false, true, vec![FailureEvent { at_s: 3600.0, asset: "sensor-x".into() }]),
        ("N10-U03", 1, false, true, vec![FailureEvent { at_s: 3600.0, asset: "bus-2".into() }]),
        ("N10-U04", 1, false, true, vec![FailureEvent { at_s: 7200.0, asset: "bus-1".into() }]),
        ("N10-OR1", 2, false, false, Vec::new()),
        ("N10-OR2", 2, false, false, vec![FailureEvent { at_s: 3600.0, asset: "robot-m1".into() }]),
        ("N10-OR3", 2, true, false, Vec::new()),
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
        let mut rows = String::from("case\tevent_sequence\tat_s\tevent_kind\tevent_origin\ttarget_asset\tresources\texecution_dependencies\ttask_resources\trestoration_targets\n");
        for event in &observed.events {
            let repair = event.running_repair.as_ref();
            rows.push_str(&format!(
                "{case}\t{}\t{:.3}\t{}\t{:?}\t{}\t{}\t{}\t{}\t{}\n",
                event.event_sequence,
                event.at_s,
                event.kind.label(),
                event.origin,
                event.target_asset.as_deref().unwrap_or(""),
                repair.map(|r| r.resources.join("+")).unwrap_or_default(),
                repair.map(|r| r.execution_dependencies.join("+")).unwrap_or_default(),
                repair
                    .map(|r| r
                        .task_resources
                        .iter()
                        .map(|e| format!("{}:{}", e.plan_index, e.resources.join("+")))
                        .collect::<Vec<_>>()
                        .join(";"))
                    .unwrap_or_default(),
                repair.map(|r| r.restoration_targets.join("+")).unwrap_or_default()
            ));
        }
        std::fs::write(out.join(format!("{case}-events.csv")), rows).unwrap();
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
