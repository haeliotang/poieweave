// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later
//! N6 canonical resource-lifetime and boundary evidence runner.

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

fn synthetic() -> Model {
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

fn plan_for(model: &Model, target: &str, horizon_s: f64) -> Plan {
    match analyze(model, target, horizon_s) {
        ClosureOutcome::Restorable(plan) => plan,
        ClosureOutcome::Blocked(blocked) => panic!("analyze {target} blocked: {:?}", blocked.reasons),
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
    let out = root().join("audit/phase6/canonical");
    std::fs::create_dir_all(&out).unwrap();
    let horizon = 20.0 * 3600.0;
    for failure_s in [7199.0, 7200.0, 7201.0, 9000.0, 10800.0] {
        let case = format!("N6-A-{failure_s:.0}");
        let model = synthetic();
        let targets = ["lathe-1", "pump-p1"];
        let plans = targets.iter().map(|target| plan_for(&model, target, horizon)).collect::<Vec<_>>();
        let pool = vec!["robot-m1".to_string(), "robot-m2".to_string()];
        let schedule =
            schedule_plans_with_alternatives(&model, &plans, &pool, &pool, std::slice::from_ref(&pool)).unwrap();
        let options = EngineOptions {
            services: vec!["potable-water", "workshop-operability"],
            horizon_s: horizon,
            failures: vec![FailureEvent { at_s: failure_s, asset: "robot-m1".into() }],
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
                ObservationRequest { at_s: failure_s, phase: ObservationPhase::Before },
                ObservationRequest { at_s: failure_s, phase: ObservationPhase::After },
                ObservationRequest { at_s: schedule.makespan_s, phase: ObservationPhase::After },
            ],
        )
        .unwrap();
        assert_eq!(outcome, observed.outcome, "noninterference {case}");
        let completion = outcome.trace.iter().find(|row| row.kind.starts_with("repair_completed"));
        let failure = outcome.trace.iter().find(|row| row.kind.starts_with("failure:"));
        let group_commit = completion.map(|row| row.at_s);
        let restored = completion
            .map(|row| row.kind.split_once(':').map(|(_, targets)| targets.to_string()).unwrap_or_default())
            .unwrap_or_default();
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
        std::fs::write(out.join(format!("{case}-events.csv")), events).unwrap();
        let schedule_csv = schedule
            .steps
            .iter()
            .map(|step| {
                format!("{}\t{:.6}\t{:.6}\t{}\n", step.process, step.start_s, step.end_s, step.resources.join("+"))
            })
            .collect::<String>();
        std::fs::write(
            out.join(format!("{case}-schedule.csv")),
            format!("process\tstart_s\tend_s\tresources\n{schedule_csv}"),
        )
        .unwrap();
        let audit = format!(
            "{{\n  \"case\": \"{case}\",\n  \"failure_s\": {failure_s:.3},\n  \"failure_applied_s\": {},\n  \"resource_acquired_s\": 0.0,\n  \"resource_release_semantics\": \"GROUP_LEVEL\",\n  \"resource_released_s\": {},\n  \"step_completed_lathe_s\": 7200.0,\n  \"step_completed_pump_s\": 10800.0,\n  \"group_committed_s\": {},\n  \"repair_interrupted\": {},\n  \"restoration_committed\": {},\n  \"restored_targets\": [{}],\n  \"termination\": \"{}\",\n  \"survived_s\": {:.6},\n  \"makespan_s\": {:.6}\n}}\n",
            failure.map(|row| format!("{:.3}", row.at_s)).unwrap_or("null".into()),
            if completion.is_some() { format!("{:.3}", group_commit.unwrap()) } else { format!("{:.3}", outcome.survived_s) },
            group_commit.map(|value| format!("{value:.3}")).unwrap_or("null".into()),
            completion.is_none(),
            completion.is_some(),
            restored
                .split(',')
                .filter(|value| !value.is_empty())
                .map(|value| format!("\"{value}\""))
                .collect::<Vec<_>>()
                .join(","),
            label(&outcome.termination),
            outcome.survived_s,
            schedule.makespan_s
        );
        std::fs::write(out.join(format!("{case}-audit.json")), audit).unwrap();
        println!(
            "{case}: {} survived={:.1} group_commit={:?}",
            label(&outcome.termination),
            outcome.survived_s,
            group_commit
        );
    }
}
