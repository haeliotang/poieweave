// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later
//! N10 canonical event-boundary evidence runner.

use std::path::{Path, PathBuf};

use poie_ontology::model::{AssetStatus, Model, Service};
use poie_viability::engine::{
    run_engine, run_engine_event_observed, EngineOptions, EngineTermination, EventObservedOutcome,
};
use poie_viability::survival::FailureEvent;

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

fn model_for(duration: f64) -> Model {
    let mut model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
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
    process.restores = vec!["a-lathe".into()];
    process.duration_s = Some(duration);
    let index = model.processes.len();
    model.processes.push(process);
    model.restorers.insert("a-lathe".into(), vec![index]);
    model.services.insert(
        "n10-service".into(),
        Service { id: "n10-service".into(), requires: vec!["a-lathe".into()], requires_any: Vec::new() },
    );
    model
}

fn write_events(out: &Path, case: &str, observed: &EventObservedOutcome) {
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
    std::fs::write(out.join("canonical").join(format!("{case}-events.jsonl")), lines).unwrap();
}

fn main() {
    let out = root().join("audit/phase12");
    std::fs::create_dir_all(out.join("canonical")).unwrap();
    let cases: [(&str, f64, f64, f64); 7] = [
        ("N10-01", 7200.0, 7199.0, 72000.0),
        ("N10-02", 7200.0, 7200.0, 72000.0),
        ("N10-03", 7200.0, 7201.0, 72000.0),
        ("N10-04", 10800.0, 10800.0, 72000.0),
        ("N10-05", 10800.0, 12000.0, 72000.0),
        ("N10-06", 7200.0, 7200.0, 7200.0),
        ("N10-00", 7200.0, -1.0, 72000.0),
    ];
    for (case, duration, failure_s, horizon) in cases {
        let model = model_for(duration);
        let failures = if failure_s >= 0.0 {
            vec![FailureEvent { at_s: failure_s, asset: "robot-m1".into() }]
        } else {
            Vec::new()
        };
        let options = EngineOptions {
            services: vec!["n10-service"],
            horizon_s: horizon,
            failures,
            resupplies: Vec::new(),
            resources: vec!["robot-m1".into()],
            alternative_pool: vec!["robot-m1".into()],
            battery: None,
            load_kw: 0.0,
        };
        let direct = run_engine(&model, &options).unwrap();
        let observed = run_engine_event_observed(&model, &options).unwrap();
        assert_eq!(direct, observed.outcome, "noninterference {case}");
        write_events(&out, case, &observed);
        let mut task_resources = Vec::new();
        if let Some(repair) = observed.events.first().and_then(|event| event.running_repair.as_ref()) {
            task_resources = repair
                .task_resources
                .iter()
                .map(|entry| format!("{}:{}", entry.plan_index, entry.resources.join("+")))
                .collect();
        }
        let completion = observed
            .events
            .iter()
            .find(|event| event.kind == poie_viability::engine::EventKind::RepairCompleted)
            .map(|event| event.at_s);
        let interruption = observed
            .events
            .iter()
            .find(|event| event.kind == poie_viability::engine::EventKind::RepairInterrupted)
            .map(|event| event.at_s);
        let summary = format!("{{\n \"case\":\"{case}\",\n \"duration_s\":{duration:.3},\n \"requested_failure_s\":{failure_s:.3},\n \"horizon_s\":{horizon:.3},\n \"completion_s\":{},\n \"interruption_s\":{},\n \"restoration_committed\":{},\n \"task_resources\":[{}],\n \"termination\":\"{}\",\n \"survived_s\":{:.3}\n}}\n",
            completion.map(|value| format!("{value:.3}")).unwrap_or_else(|| "null".into()),
            interruption.map(|value| format!("{value:.3}")).unwrap_or_else(|| "null".into()),
            completion.is_some(),
            task_resources.iter().map(|value| format!("\"{value}\"")).collect::<Vec<_>>().join(","),
            label(&observed.outcome.termination), observed.outcome.survived_s);
        std::fs::write(out.join("canonical").join(format!("{case}-summary.json")), summary).unwrap();
        println!(
            "{case}: completion={completion:?} interruption={interruption:?} {}",
            label(&observed.outcome.termination)
        );
    }
}
