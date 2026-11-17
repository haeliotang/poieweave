// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later
//! N6-B parameterized canonical scheduling evidence runner.

use std::path::{Path, PathBuf};

use poie_ontology::model::{AssetStatus, Model, Service};
use poie_viability::closure::{analyze, ClosureOutcome, Plan};
use poie_viability::engine::{run_engine, EngineOptions, EngineTermination};
use poie_viability::scheduler::schedule_plans_with_alternatives;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[derive(Clone, Copy)]
struct Case {
    id: &'static str,
    tasks: usize,
    durations: [f64; 3],
    robots: usize,
    reversed_priority: bool,
}

fn task_model(tasks: usize, durations: [f64; 3]) -> Model {
    let mut model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let robot = model.assets.get("robot-m1").unwrap().clone();
    let mut second = robot.clone();
    second.id = "robot-m2".into();
    model.assets.insert("robot-m2".into(), second);
    let pump = model.assets.get("pump-p1").unwrap().clone();
    let process_template = model.processes.iter().find(|process| process.id == "replace-sleeve-p1").unwrap().clone();
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
        let mut process = process_template.clone();
        process.id = format!("repair-{}", target);
        process.inputs.clear();
        process.outputs.clear();
        process.occupies = vec!["robot-m1".into()];
        process.restores = vec![target.into()];
        process.requires_capabilities.clear();
        process.duration_s = Some(durations[index]);
        let process_index = model.processes.len();
        model.processes.push(process);
        model.restorers.entry(target.into()).or_default().push(process_index);
        requires.push(target.into());
    }
    model.services.insert(
        "n6-tasks".into(),
        Service { id: "n6-tasks".into(), requires: requires.clone(), requires_any: Vec::new() },
    );
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
    let cases = [
        Case { id: "N6-B00", tasks: 2, durations: [7200.0, 10800.0, 0.0], robots: 2, reversed_priority: false },
        Case { id: "N6-B01", tasks: 2, durations: [5000.0, 10800.0, 0.0], robots: 2, reversed_priority: false },
        Case { id: "N6-B02", tasks: 2, durations: [7200.0, 9000.0, 0.0], robots: 2, reversed_priority: false },
        Case { id: "N6-B03", tasks: 2, durations: [5000.0, 9000.0, 0.0], robots: 2, reversed_priority: false },
        Case { id: "N6-B04", tasks: 1, durations: [7200.0, 0.0, 0.0], robots: 1, reversed_priority: false },
        Case { id: "N6-B05", tasks: 3, durations: [7200.0, 10800.0, 6000.0], robots: 2, reversed_priority: false },
        Case { id: "N6-B06", tasks: 2, durations: [7200.0, 10800.0, 0.0], robots: 1, reversed_priority: true },
        Case { id: "N6-B07", tasks: 2, durations: [7200.0, 10800.0, 0.0], robots: 1, reversed_priority: false },
    ];
    let out = root().join("audit/phase6");
    std::fs::create_dir_all(out.join("canonical")).unwrap();
    std::fs::create_dir_all(out.join("contracts")).unwrap();
    for case in cases {
        let horizon = 20.0 * 3600.0;
        let model = task_model(case.tasks, case.durations);
        let targets = ["a-lathe", "b-pump", "c-filter"];
        let plans = (0..case.tasks).map(|index| plan_for(&model, targets[index], horizon)).collect::<Vec<_>>();
        let pool: Vec<String> = (1..=case.robots).map(|index| format!("robot-m{index}")).collect();
        let schedule =
            schedule_plans_with_alternatives(&model, &plans, &pool, &pool, std::slice::from_ref(&pool)).unwrap();
        let options = EngineOptions {
            services: vec!["n6-tasks"],
            horizon_s: horizon,
            failures: Vec::new(),
            resupplies: Vec::new(),
            resources: pool.clone(),
            alternative_pool: pool.clone(),
            battery: None,
            load_kw: 0.0,
        };
        let outcome = run_engine(&model, &options).unwrap();
        let mut csv = String::from("task_index\tstart_s\tend_s\tresources\n");
        for (index, step) in schedule.steps.iter().enumerate() {
            csv.push_str(&format!("{}\t{:.6}\t{:.6}\t{}\n", index, step.start_s, step.end_s, step.resources.join("+")));
        }
        std::fs::write(out.join("canonical").join(format!("{}-schedule.csv", case.id)), csv).unwrap();
        let mut contract = String::new();
        contract.push_str(&format!("taskCount={}\n", case.tasks));
        for index in 0..3 {
            let priority = if case.reversed_priority && index == 0 {
                2
            } else if case.reversed_priority && index == 1 {
                1
            } else {
                index + 1
            };
            let mask = if index < case.tasks { (1 << case.robots) - 1 } else { 0 };
            contract.push_str(&format!(
                "t{}Duration_s={}\nt{}Priority={}\nt{}RobotMask={}\nt{}RequiresTool=0\n",
                index + 1,
                case.durations[index],
                index + 1,
                priority,
                index + 1,
                mask,
                index + 1
            ));
        }
        contract.push_str(&format!(
            "robotCount={}\nfailureTime_s=-1\nfailureKind=0\nfailureIndex=0\nhorizon_s=72000\n",
            case.robots
        ));
        std::fs::write(out.join("contracts").join(format!("{}.contract", case.id)), contract).unwrap();
        let completion = outcome.trace.iter().find(|row| row.kind.starts_with("repair_completed"));
        let restored = completion
            .map(|row| row.kind.split_once(':').map(|(_, targets)| targets.to_string()).unwrap_or_default())
            .unwrap_or_default();
        let summary = format!(
            "{{\n  \"case\": \"{}\",\n  \"task_count\": {},\n  \"durations\": {:?},\n  \"robots\": {},\n  \"makespan_s\": {:.6},\n  \"termination\": \"{}\",\n  \"restored_targets\": [{}],\n  \"schedule\": [{}]\n}}\n",
            case.id,
            case.tasks,
            &case.durations[..case.tasks],
            case.robots,
            schedule.makespan_s,
            label(&outcome.termination),
            restored.split(',').filter(|v| !v.is_empty()).map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(","),
            schedule.steps.iter().map(|step| format!(
                "{{\"start_s\":{:.3},\"end_s\":{:.3},\"resources\":[{}]}}",
                step.start_s,
                step.end_s,
                step.resources.iter().map(|r| format!("\"{r}\"")).collect::<Vec<_>>().join(",")
            )).collect::<Vec<_>>().join(",")
        );
        std::fs::write(out.join("canonical").join(format!("{}-summary.json", case.id)), summary).unwrap();
        println!("{}: makespan={:.1} tasks={}", case.id, schedule.makespan_s, case.tasks);
    }
}
