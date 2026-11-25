// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later
use poie_ontology::model::{AssetStatus, Model, Service};
use poie_viability::engine::{
    resume_engine_with_authorization, run_engine_with_repair_group_event_observed,
    run_engine_with_repair_group_to_checkpoint, EngineOptions, ExplicitRepairGroup, ExplicitRepairTask,
    RepairLifecycleMode, ReplanAuthorization, ReplanProposal,
};
use poie_viability::survival::FailureEvent;
use std::path::Path;

fn model() -> Model {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    let robot = model.assets.get("robot-m1").unwrap().clone();
    let mut second = robot.clone();
    second.id = "robot-m2".into();
    model.assets.insert("robot-m2".into(), second);
    for extra in ["sensor-x", "bus-2"] {
        let mut a = robot.clone();
        a.id = extra.into();
        a.kind = "sensor".into();
        a.requires.clear();
        a.requires_any.clear();
        a.provides.clear();
        model.assets.insert(extra.into(), a);
    }
    let mut bus2 = model.assets.get("bus-1").unwrap().clone();
    bus2.id = "bus-2".into();
    bus2.requires.clear();
    bus2.requires_any.clear();
    model.assets.insert("bus-2".into(), bus2);
    if let Some(r2) = model.assets.get_mut("robot-m2") {
        r2.requires = vec!["bus-2".into()];
        r2.requires_any.clear();
        r2.provides.push("n16-robot-b".into());
    }
    let pump = model.assets.get("pump-p1").unwrap().clone();
    let mut lathe = pump;
    lathe.id = "a-lathe".into();
    lathe.status = AssetStatus::Failed;
    lathe.requires.clear();
    lathe.requires_any.clear();
    lathe.provides.clear();
    model.assets.insert("a-lathe".into(), lathe);
    let template = model.processes.iter().find(|p| p.id == "replace-sleeve-p1").unwrap().clone();
    let mut a = template.clone();
    a.id = "n10-repair-a-lathe".into();
    a.inputs.clear();
    a.outputs.clear();
    a.requires_capabilities.clear();
    a.occupies = vec!["robot-m2".into()];
    a.restores = vec!["a-lathe".into()];
    a.duration_s = Some(7200.0);
    let ai = model.processes.len();
    model.processes.push(a);
    model.restorers.insert("a-lathe".into(), vec![ai]);
    let mut b = template;
    b.id = "n14-task-b".into();
    b.inputs.clear();
    b.outputs.clear();
    b.requires_capabilities.clear();
    b.occupies = vec!["robot-m1".into()];
    b.restores = vec!["a-lathe".into()];
    b.duration_s = Some(10800.0);
    model.processes.push(b);
    model
        .services
        .insert("svc".into(), Service { id: "svc".into(), requires: vec!["a-lathe".into()], requires_any: Vec::new() });
    model
}
fn group() -> ExplicitRepairGroup {
    ExplicitRepairGroup {
        group_id: "g".into(),
        restoration_target: "a-lathe".into(),
        lifecycle: RepairLifecycleMode::TaskRelease,
        tasks: vec![
            ExplicitRepairTask {
                task_id: "A".into(),
                process: "n10-repair-a-lathe".into(),
                duration_s: 7200.0,
                robot_alternatives: vec!["robot-m2".into()],
            },
            ExplicitRepairTask {
                task_id: "B".into(),
                process: "n14-task-b".into(),
                duration_s: 10800.0,
                robot_alternatives: vec!["robot-m1".into()],
            },
        ],
    }
}
#[test]
fn native_checkpoint_resume_replans_unfinished_b_only() {
    let model = model();
    let options = EngineOptions {
        services: vec!["svc"],
        horizon_s: 72000.0,
        failures: vec![FailureEvent { at_s: 9000.0, asset: "robot-m1".into() }],
        resupplies: vec![],
        resources: vec!["robot-m1".into(), "robot-m2".into()],
        alternative_pool: vec!["robot-m1".into(), "robot-m2".into()],
        battery: None,
        load_kw: 0.0,
    };
    let observed = run_engine_with_repair_group_event_observed(&model, &options, &group()).unwrap();
    for e in &observed.events {
        if let Some(r) = &e.running_repair {
            println!("event {} {:?} {:?}", e.kind.label(), e.at_s, r.task_resources);
        }
    }
    let (outcome, checkpoint) =
        run_engine_with_repair_group_to_checkpoint(&model, &options, &group(), "exec-1").unwrap();
    for tr in &outcome.trace {
        println!("{} {:?}", tr.at_s, tr.kind);
    }
    println!(
        "outcome={:?} cp={:?}",
        outcome.termination,
        checkpoint.as_ref().map(|c| (&c.at_s, &c.completed_tasks, &c.unfinished_tasks))
    );
    let cp = checkpoint.expect("checkpoint");
    assert!(matches!(outcome.termination, poie_viability::engine::EngineTermination::Blocked(_)));
    assert_eq!(cp.at_s, 9000.0);
    assert_eq!(cp.completed_tasks, vec!["A".to_string()]);
    assert_eq!(cp.unfinished_tasks, vec!["B".to_string()]);
    assert_eq!(cp.asset_statuses.get("robot-m1"), Some(&AssetStatus::Failed));
    let mut proposal = ReplanProposal { proposal_id: "p".into(), task_robots: Default::default() };
    proposal.task_robots.insert("B".into(), vec!["robot-m2".into()]);
    let auth = ReplanAuthorization::issue("auth-1", &cp, &proposal, "ONE_SHOT_REPLAN");
    let resumed = resume_engine_with_authorization(&model, &options, &group(), &cp, &auth, &proposal).unwrap();
    assert_eq!(resumed.termination, poie_viability::engine::EngineTermination::CompletedHorizon);
    assert_eq!(resumed.survived_s, 19800.0);
    assert_eq!(cp.asset_statuses.get("robot-m1"), Some(&AssetStatus::Failed));
    assert_eq!(
        resumed.final_inventory.get("electric-energy").copied().unwrap_or(0.0),
        cp.inventory.get("electric-energy").copied().unwrap_or(0.0)
    );
}
