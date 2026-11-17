// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later
//! N6-C/D dependency and event-boundary evidence runner.

use std::path::{Path, PathBuf};

use poie_ontology::model::{AssetStatus, Model, Service};
use poie_viability::closure::{analyze, ClosureOutcome, Plan};
use poie_viability::engine::{
    run_engine, run_engine_observed, EngineOptions, EngineTermination, ObservationPhase, ObservationRequest,
};
use poie_viability::scheduler::schedule_plans_with_alternatives;
use poie_viability::survival::FailureEvent;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn model_with_tasks(tasks: usize, durations: &[f64], tool: bool, capability: bool) -> Model {
    let mut model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let robot = model.assets.get("robot-m1").unwrap().clone();
    let mut second = robot.clone();
    second.id = "robot-m2".into();
    model.assets.insert("robot-m2".into(), second);
    if tool {
        let mut tool = robot.clone();
        tool.id = "tool-t1".into();
        tool.kind = "tool".into();
        tool.status = AssetStatus::Available;
        tool.requires.clear();
        tool.requires_any.clear();
        tool.provides.clear();
        model.assets.insert("tool-t1".into(), tool);
    }
    let pump = model.assets.get("pump-p1").unwrap().clone();
    let template = model.processes.iter().find(|process| process.id == "replace-sleeve-p1").unwrap().clone();
    let names = ["a-lathe", "b-pump", "c-filter"];
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
        process.id = format!("repair-{target}");
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
        process.restores = vec![target.into()];
        process.duration_s = Some(durations[index]);
        let process_index = model.processes.len();
        model.processes.push(process);
        model.restorers.entry(target.into()).or_default().push(process_index);
        requires.push(target.into());
    }
    if capability {
        for provider in ["robot-m1", "robot-m2"] {
            model.providers.entry("sleeve-replacement".into()).or_default().push(provider.into());
        }
    }
    model.services.insert("n6-tasks".into(), Service { id: "n6-tasks".into(), requires, requires_any: Vec::new() });
    model
}

fn plan_for(model: &Model, target: &str, horizon: f64) -> Plan {
    match analyze(model, target, horizon) {
        ClosureOutcome::Restorable(plan) => plan,
        ClosureOutcome::Blocked(blocked) => panic!("blocked: {:?}", blocked.reasons),
    }
}
fn label(term: &EngineTermination) -> String {
    match term {
        EngineTermination::CompletedHorizon => "completed".into(),
        EngineTermination::Blocked(reason) => format!("blocked:{reason}"),
        EngineTermination::IncompleteAtHorizon(reason) => format!("incomplete:{reason}"),
    }
}

fn main() {
    let out = root().join("audit/phase6");
    std::fs::create_dir_all(out.join("canonical")).unwrap();
    let cases = [
        ("N6-C-tool-1", 1usize, [10800.0, 0.0, 0.0], true, false, Some(("tool-t1", 1.0)), 72000.0),
        ("N6-C-tool-9000", 1, [10800.0, 0.0, 0.0], true, false, Some(("tool-t1", 9000.0)), 72000.0),
        ("N6-C-tool-10800", 1, [10800.0, 0.0, 0.0], true, false, Some(("tool-t1", 10800.0)), 72000.0),
        ("N6-C-tool-12000", 1, [10800.0, 0.0, 0.0], true, false, Some(("tool-t1", 12000.0)), 72000.0),
        ("N6-C-bus-9000", 1, [10800.0, 0.0, 0.0], false, true, Some(("bus-1", 9000.0)), 72000.0),
        ("N6-C-bus-10800", 1, [10800.0, 0.0, 0.0], false, true, Some(("bus-1", 10800.0)), 72000.0),
        ("N6-D-same-10800", 2, [7200.0, 10800.0, 0.0], false, false, Some(("robot-m1", 10800.0)), 72000.0),
        ("N6-D-horizon-completion", 1, [10800.0, 0.0, 0.0], false, false, Some(("robot-m1", 10800.0)), 10800.0),
        ("N6-D-horizon-failure", 1, [7200.0, 0.0, 0.0], false, false, Some(("robot-m1", 10800.0)), 10800.0),
    ];
    for (id, tasks, durations, tool, capability, failure, horizon) in cases {
        let model = model_with_tasks(tasks, &durations, tool, capability);
        let targets = ["a-lathe", "b-pump", "c-filter"];
        let plans = (0..tasks).map(|index| plan_for(&model, targets[index], horizon)).collect::<Vec<_>>();
        let pool = vec!["robot-m1".to_string(), "robot-m2".to_string()];
        let mut schedule_pool = pool.clone();
        if tool {
            schedule_pool.push("tool-t1".into());
        }
        let schedule =
            schedule_plans_with_alternatives(&model, &plans, &schedule_pool, &pool, std::slice::from_ref(&pool))
                .unwrap();
        let failures =
            failure.into_iter().map(|(asset, at_s)| FailureEvent { at_s, asset: asset.into() }).collect::<Vec<_>>();
        let options = EngineOptions {
            services: vec!["n6-tasks"],
            horizon_s: horizon,
            failures,
            resupplies: Vec::new(),
            resources: pool.clone(),
            alternative_pool: pool.clone(),
            battery: None,
            load_kw: 0.0,
        };
        let outcome = run_engine(&model, &options).unwrap();
        let observed = run_engine_observed(
            &model,
            &options,
            &[
                ObservationRequest { at_s: schedule.makespan_s, phase: ObservationPhase::Before },
                ObservationRequest { at_s: schedule.makespan_s, phase: ObservationPhase::After },
                ObservationRequest { at_s: failure.map(|(_, t)| t).unwrap_or(0.0), phase: ObservationPhase::After },
            ],
        )
        .unwrap();
        assert_eq!(outcome, observed.outcome, "noninterference {id}");
        let completion = outcome.trace.iter().find(|row| row.kind.starts_with("repair_completed"));
        let failed = outcome.trace.iter().find(|row| row.kind.starts_with("failure:"));
        let mut events = String::from("at_s\tkind\tplans\tresources\n");
        for row in &outcome.trace {
            events.push_str(&format!(
                "{:.6}\t{}\t{}\t{}\n",
                row.at_s,
                row.kind,
                row.plans.join("+"),
                row.resources.join("+")
            ));
        }
        std::fs::write(out.join("canonical").join(format!("{id}-events.csv")), events).unwrap();
        let mut schedule_csv = String::from("start_s\tend_s\tresources\n");
        for step in &schedule.steps {
            schedule_csv.push_str(&format!("{:.6}\t{:.6}\t{}\n", step.start_s, step.end_s, step.resources.join("+")));
        }
        std::fs::write(out.join("canonical").join(format!("{id}-schedule.csv")), schedule_csv).unwrap();
        let restored = completion
            .map(|row| row.kind.split_once(':').map(|(_, t)| t.to_string()).unwrap_or_default())
            .unwrap_or_default();
        let audit = format!(
            "{{\n \"case\":\"{id}\",\n \"tasks\":{tasks},\n \"tool_required\":{tool},\n \"upstream_required\":{capability},\n \"horizon_s\":{horizon},\n \"makespan_s\":{:.6},\n \"termination\":\"{}\",\n \"survived_s\":{:.6},\n \"failure_applied_s\":{},\n \"repair_completed_s\":{},\n \"repair_interrupted\":{},\n \"restored_targets\":[{}]\n}}\n",
            schedule.makespan_s, label(&outcome.termination), outcome.survived_s,
            failed.map(|row| format!("{:.3}", row.at_s)).unwrap_or("null".into()),
            completion.map(|row| format!("{:.3}", row.at_s)).unwrap_or("null".into()),
            completion.is_none() && outcome.survived_s < horizon,
            restored.split(',').filter(|v| !v.is_empty()).map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(",")
        );
        std::fs::write(out.join("canonical").join(format!("{id}-audit.json")), audit).unwrap();
        println!(
            "{id}: {} failure={:?} completion={:?}",
            label(&outcome.termination),
            failed.map(|r| r.at_s),
            completion.map(|r| r.at_s)
        );
    }
}
