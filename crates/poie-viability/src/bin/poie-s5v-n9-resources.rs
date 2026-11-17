// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later
//! N9 canonical resource-causal trajectory evidence runner.

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

fn model_for(
    tasks: usize,
    durations: &[f64],
    robots: usize,
    tool: bool,
    capability: bool,
    preferred_robot: Option<usize>,
) -> Model {
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
        tool_asset.status = AssetStatus::Available;
        tool_asset.requires.clear();
        tool_asset.requires_any.clear();
        tool_asset.provides.clear();
        model.assets.insert("tool-t1".into(), tool_asset);
    }
    let pump = model.assets.get("pump-p1").unwrap().clone();
    let template = model.processes.iter().find(|process| process.id == "replace-sleeve-p1").unwrap().clone();
    let names = ["a-lathe", "b-pump"];
    let mut requires = Vec::new();
    for index in 0..tasks {
        let target = names[index];
        let mut asset = pump.clone();
        asset.id = target.into();
        asset.status = AssetStatus::Failed;
        asset.requires.clear();
        asset.requires_any.clear();
        asset.provides.clear();
        model.assets.insert(target.into(), asset);
        let mut process = template.clone();
        process.id = format!("n9-repair-{target}");
        process.inputs.clear();
        process.outputs.clear();
        process.requires_capabilities.clear();
        let robot_index = preferred_robot.unwrap_or(if robots > 1 { index + 1 } else { 1 });
        process.occupies = vec![format!("robot-m{robot_index}")];
        if tool {
            process.occupies.push("tool-t1".into());
        }
        if capability {
            process.requires_capabilities = vec!["sleeve-replacement".into()];
        }
        process.restores = vec![target.into()];
        process.duration_s = Some(durations[index]);
        let process_index = model.processes.len();
        model.processes.push(process);
        model.restorers.entry(target.into()).or_default().push(process_index);
        requires.push(target.into());
    }
    if robots > 1 {
        for provider in ["robot-m1", "robot-m2"] {
            for capability in ["sleeve-replacement", "lathe-repair", "machine-tending", "material-handling"] {
                model.providers.entry(capability.into()).or_default().push(provider.into());
            }
        }
    }
    model.services.insert("n9-service".into(), Service { id: "n9-service".into(), requires, requires_any: Vec::new() });
    model
}

fn write_case(out: &Path, case: &str, model: &Model, options: &EngineOptions<'_>) -> (usize, String) {
    let direct = run_engine(model, options).unwrap();
    let observed = run_engine_event_observed(model, options).unwrap();
    assert_eq!(direct, observed.outcome, "noninterference {case}");
    write_events(out, case, &observed);
    let mut selected = Vec::new();
    let mut dependencies = Vec::new();
    let mut targets = Vec::new();
    let mut task_order: Vec<String> = Vec::new();
    let mut witness = Vec::new();
    if let Some(repair) = observed.events.first().and_then(|event| event.running_repair.as_ref()) {
        selected = repair.resources.clone();
        dependencies = repair.execution_dependencies.clone();
        targets = repair.restoration_targets.clone();
        task_order = repair.plans.clone();
        witness = repair
            .witness
            .iter()
            .map(|entry| format!("{}:{}:{}", entry.capability, entry.provider, entry.dependencies.join("|")))
            .collect();
    }
    let restoration_committed =
        observed.events.iter().any(|event| event.kind == poie_viability::engine::EventKind::RepairCompleted);
    let repair_completed_s = observed
        .events
        .iter()
        .find(|event| event.kind == poie_viability::engine::EventKind::RepairCompleted)
        .map(|event| event.at_s);
    let summary = format!(
        "{{\n \"case\":\"{case}\",\n \"restoration_committed\":{},\n \"repair_completed_s\":{},\n \"selected_resources\":[{}],\n \"execution_dependencies\":[{}],\n \"restoration_targets\":[{}],\n \"witness\":[{}],\n \"task_order\":[{}],\n \"termination\":\"{}\",\n \"survived_s\":{:.3},\n \"event_count\":{},\n \"resource_release_observed\":false,\n \"release_note\":\"RunTerminated is not a native resource release event\",\n \"observation_source\":\"NATIVE_CANONICAL_RUNTIME\"\n}}\n",
        restoration_committed,
        repair_completed_s.map(|value| format!("{value:.3}")).unwrap_or_else(|| "null".into()),
        selected.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(","),
        dependencies.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(","),
        targets.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(","),
        witness.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(","),
        task_order.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(","),
        label(&observed.outcome.termination), observed.outcome.survived_s, observed.events.len()
    );
    std::fs::write(out.join("canonical").join(format!("{case}-summary.json")), summary).unwrap();
    (observed.events.len(), label(&observed.outcome.termination))
}

fn write_events(out: &Path, case: &str, observed: &EventObservedOutcome) {
    let mut rows = String::from("case\tevent_sequence\tevent_batch_sequence\tat_s\tevent_kind\tevent_origin\ttarget_asset\tevent_status\tresources\texecution_dependencies\trestoration_targets\twitness\tterm\n");
    for event in &observed.events {
        let repair = event.running_repair.as_ref();
        rows.push_str(&format!(
            "{case}\t{}\t{}\t{:.3}\t{}\t{:?}\t{}\t{:?}\t{}\t{}\t{}\t{}\t{}\n",
            event.event_sequence,
            event.event_batch_sequence,
            event.at_s,
            event.kind.label(),
            event.origin,
            event.target_asset.as_deref().unwrap_or(""),
            event.status,
            repair.map(|r| r.resources.join("+")).unwrap_or_default(),
            repair.map(|r| r.execution_dependencies.join("+")).unwrap_or_default(),
            repair.map(|r| r.restoration_targets.join("+")).unwrap_or_default(),
            repair
                .map(|r| r
                    .witness
                    .iter()
                    .map(|w| format!("{}:{}:{}", w.capability, w.provider, w.dependencies.join("|")))
                    .collect::<Vec<_>>()
                    .join(";"))
                .unwrap_or_default(),
            label(&observed.outcome.termination)
        ));
    }
    std::fs::write(out.join("canonical").join(format!("{case}-events.csv")), rows).unwrap();
}

fn options(resources: Vec<String>, failures: Vec<FailureEvent>) -> EngineOptions<'static> {
    EngineOptions {
        services: vec!["n9-service"],
        horizon_s: 20.0 * 3600.0,
        failures,
        resupplies: Vec::new(),
        resources: resources.clone(),
        alternative_pool: resources,
        battery: None,
        load_kw: 0.0,
    }
}

fn main() {
    let out = root().join("audit/phase9");
    std::fs::create_dir_all(out.join("canonical")).unwrap();
    let pools1 = vec!["robot-m1".to_string()];
    let pools2 = vec!["robot-m1".to_string(), "robot-m2".to_string()];
    let cases = [
        ("N9-00", 1usize, [7200.0, 0.0], 1usize, false, false, Vec::new()),
        ("N9-01", 2, [7200.0, 10800.0], 1, false, false, Vec::new()),
        ("N9-02", 2, [7200.0, 10800.0], 2, false, false, Vec::new()),
        ("N9-03", 1, [10800.0, 0.0], 1, false, false, vec![FailureEvent { at_s: 7200.0, asset: "robot-m1".into() }]),
        ("N9-04", 1, [10800.0, 0.0], 2, false, false, vec![FailureEvent { at_s: 7200.0, asset: "robot-m2".into() }]),
        ("N9-05", 1, [10800.0, 0.0], 1, true, false, vec![FailureEvent { at_s: 9000.0, asset: "tool-t1".into() }]),
        ("N9-06", 1, [10800.0, 0.0], 1, false, true, vec![FailureEvent { at_s: 9000.0, asset: "bus-1".into() }]),
        ("N9-11", 2, [7200.0, 10800.0], 1, false, false, Vec::new()),
    ];
    for (case, tasks, durations, robots, tool, capability, failures) in cases {
        let preferred = None;
        let model = model_for(tasks, &durations, robots, tool, capability, preferred);
        let pool = if robots > 1 { pools2.clone() } else { pools1.clone() };
        let opts = options(pool, failures);
        let (count, termination) = write_case(&out, case, &model, &opts);
        println!("{case}: {count} events {termination}");
    }
}
