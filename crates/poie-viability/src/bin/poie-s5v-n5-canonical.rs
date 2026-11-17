// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later
//! N5 canonical independent-scheduling evidence runner.

use std::path::{Path, PathBuf};

use poie_ontology::model::{AssetStatus, Model};
use poie_viability::closure::{analyze, ClosureOutcome, Plan};
use poie_viability::engine::{
    run_engine, run_engine_observed, EngineOptions, EngineTermination, ObservationPhase, ObservationRequest,
};
use poie_viability::scheduler::schedule_plans_with_alternatives;
use poie_viability::survival::FailureEvent;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[derive(Clone, Copy)]
struct Case {
    id: &'static str,
    lathe: bool,
    robots: usize,
    failure: Option<(&'static str, f64)>,
}

fn synthetic(lathe_failed: bool) -> Model {
    let mut model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    if lathe_failed {
        model.assets.get_mut("lathe-1").unwrap().status = AssetStatus::Failed;
    }
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

fn plan_for(model: &Model, target: &str, horizon_s: f64) -> Plan {
    match analyze(model, target, horizon_s) {
        ClosureOutcome::Restorable(plan) => plan,
        ClosureOutcome::Blocked(blocked) => panic!("analyze {target} blocked: {:?}", blocked.reasons),
    }
}

fn termination(term: &EngineTermination) -> String {
    match term {
        EngineTermination::CompletedHorizon => "completed".into(),
        EngineTermination::Blocked(reason) => format!("blocked:{reason}"),
        EngineTermination::IncompleteAtHorizon(reason) => format!("incomplete:{reason}"),
    }
}

fn main() {
    let cases = [
        Case { id: "N5-00", lathe: false, robots: 1, failure: None },
        Case { id: "N5-01", lathe: true, robots: 1, failure: None },
        Case { id: "N5-02", lathe: true, robots: 2, failure: None },
        Case { id: "N5-03", lathe: true, robots: 2, failure: Some(("robot-m1", 7200.0)) },
        Case { id: "N5-04", lathe: false, robots: 2, failure: Some(("robot-m2", 7200.0)) },
    ];
    let out = root().join("audit/phase5/canonical");
    std::fs::create_dir_all(&out).unwrap();
    for case in cases {
        let model = synthetic(case.lathe);
        let horizon = 20.0 * 3600.0;
        let mut targets = Vec::new();
        if case.lathe {
            targets.push("lathe-1");
        }
        targets.push("pump-p1");
        let plans = targets.iter().map(|target| plan_for(&model, target, horizon)).collect::<Vec<_>>();
        let pool: Vec<String> = (1..=case.robots).map(|index| format!("robot-m{index}")).collect();
        let schedule =
            schedule_plans_with_alternatives(&model, &plans, &pool, &pool, std::slice::from_ref(&pool)).unwrap();
        let failures = case
            .failure
            .into_iter()
            .map(|(asset, at_s)| FailureEvent { at_s, asset: asset.into() })
            .collect::<Vec<_>>();
        let options = EngineOptions {
            services: vec!["potable-water", "workshop-operability"],
            horizon_s: horizon,
            failures,
            resupplies: Vec::new(),
            resources: pool.clone(),
            alternative_pool: pool.clone(),
            battery: None,
            load_kw: 0.0,
        };
        let outcome = run_engine(&model, &options).unwrap();
        let requests = [
            ObservationRequest { at_s: 0.0, phase: ObservationPhase::After },
            ObservationRequest { at_s: schedule.makespan_s.min(7200.0), phase: ObservationPhase::After },
            ObservationRequest { at_s: schedule.makespan_s, phase: ObservationPhase::Before },
            ObservationRequest { at_s: schedule.makespan_s, phase: ObservationPhase::After },
        ];
        let observed = run_engine_observed(&model, &options, &requests).unwrap();
        assert_eq!(outcome, observed.outcome, "native observation noninterference failed for {}", case.id);
        let mut schedule_csv = String::from("task_index\tprocess\ttarget\tstart_s\tend_s\tresources\n");
        for (index, step) in schedule.steps.iter().enumerate() {
            let target = targets.get(step.plan_index).copied().unwrap_or("?");
            schedule_csv.push_str(&format!(
                "{}\t{}\t{}\t{:.6}\t{:.6}\t{}\n",
                index,
                step.process,
                target,
                step.start_s,
                step.end_s,
                step.resources.join("+")
            ));
        }
        std::fs::write(out.join(format!("{}-schedule.csv", case.id)), schedule_csv).unwrap();
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
        std::fs::write(out.join(format!("{}-events.csv", case.id)), events).unwrap();
        let completion = outcome.trace.iter().find(|row| row.kind.starts_with("repair_completed"));
        let restored = completion
            .map(|row| row.kind.split_once(':').map(|(_, targets)| targets.to_string()).unwrap_or_default())
            .unwrap_or_default();
        let summary = format!(
            "{{\n  \"case\": \"{}\",\n  \"termination\": \"{}\",\n  \"survived_s\": {:.6},\n  \"makespan_s\": {:.6},\n  \"interrupted\": {},\n  \"restored_targets\": [{}],\n  \"schedule\": [{}]\n}}\n",
            case.id,
            termination(&outcome.termination),
            outcome.survived_s,
            schedule.makespan_s,
            completion.is_none() && outcome.survived_s < horizon,
            restored
                .split(',')
                .filter(|value| !value.is_empty())
                .map(|value| format!("\"{value}\""))
                .collect::<Vec<_>>()
                .join(","),
            schedule
                .steps
                .iter()
                .map(|step| format!(
                    "{{\"process\":\"{}\",\"start_s\":{:.3},\"end_s\":{:.3},\"resources\":[{}]}}",
                    step.process,
                    step.start_s,
                    step.end_s,
                    step.resources.iter().map(|resource| format!("\"{resource}\"")).collect::<Vec<_>>().join(",")
                ))
                .collect::<Vec<_>>()
                .join(",")
        );
        std::fs::write(out.join(format!("{}-summary.json", case.id)), summary).unwrap();
        println!(
            "{}: {} survived={:.1} makespan={:.1} interrupted={}",
            case.id,
            termination(&outcome.termination),
            outcome.survived_s,
            schedule.makespan_s,
            completion.is_none() && outcome.survived_s < horizon
        );
    }
}
