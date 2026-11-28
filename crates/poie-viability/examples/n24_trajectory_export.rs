// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later
use poie_ontology::model::{AssetStatus, Model, Service};
use poie_viability::engine::{
    resume_engine_with_authorization_event_observed, run_engine_with_repair_group_to_checkpoint_event_observed,
    verify_trusted_checkpoint, EngineOptions, ExplicitRepairGroup, ExplicitRepairTask, RepairLifecycleMode,
    ReplanAuthorization, ReplanProposal,
};
use poie_viability::survival::FailureEvent;
use std::collections::BTreeSet;
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
    let mut third = robot.clone();
    third.id = "robot-m3".into();
    model.assets.insert("robot-m3".into(), third);
    let mut bus3 = model.assets.get("bus-1").unwrap().clone();
    bus3.id = "bus-3".into();
    bus3.requires.clear();
    bus3.requires_any.clear();
    model.assets.insert("bus-3".into(), bus3);
    if let Some(r3) = model.assets.get_mut("robot-m3") {
        r3.requires = vec!["bus-3".into()];
        r3.requires_any.clear();
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
    let mut b = template.clone();
    b.id = "n14-task-b".into();
    b.inputs.clear();
    b.outputs.clear();
    b.requires_capabilities.clear();
    b.occupies = vec!["robot-m1".into()];
    b.restores = vec!["a-lathe".into()];
    b.duration_s = Some(10800.0);
    model.processes.push(b);
    let mut c = template.clone();
    c.id = "n24-task-c".into();
    c.inputs.clear();
    c.outputs.clear();
    c.requires_capabilities.clear();
    c.occupies = vec!["robot-m2".into()];
    c.restores = vec!["a-lathe".into()];
    c.duration_s = Some(3600.0);
    model.processes.push(c);
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
            ExplicitRepairTask {
                task_id: "C".into(),
                process: "n24-task-c".into(),
                duration_s: 3600.0,
                robot_alternatives: vec!["robot-m2".into()],
            },
        ],
    }
}

fn main() {
    let model = model();
    let options = EngineOptions {
        services: vec!["svc"],
        horizon_s: 72000.0,
        failures: vec![FailureEvent { at_s: 9000.0, asset: "robot-m1".into() }],
        resupplies: vec![],
        resources: vec!["robot-m1".into(), "robot-m2".into(), "robot-m3".into()],
        alternative_pool: vec!["robot-m1".into(), "robot-m2".into(), "robot-m3".into()],
        battery: None,
        load_kw: 0.0,
    };
    let group = group();
    let (_, checkpoint, initial_events) =
        run_engine_with_repair_group_to_checkpoint_event_observed(&model, &options, &group, "n24-trajectory").unwrap();
    let checkpoint = checkpoint.expect("checkpoint");
    let mut trusted = BTreeSet::new();
    trusted.insert(checkpoint.compute_state_hash());
    verify_trusted_checkpoint(&checkpoint, &trusted).unwrap();
    let mut task_robots = std::collections::BTreeMap::new();
    task_robots.insert("B".to_string(), vec!["robot-m3".to_string()]);
    task_robots.insert("C".to_string(), vec!["robot-m2".to_string()]);
    let checkpoint_hash = checkpoint.compute_state_hash();
    let mut proposal = ReplanProposal {
        proposal_id: "n24-two-task".into(),
        execution_id: checkpoint.execution_id.clone(),
        checkpoint_hash: checkpoint_hash.clone(),
        group_id: checkpoint.group_id.clone(),
        unfinished_task_ids: checkpoint.unfinished_tasks.clone(),
        task_robots: task_robots.clone(),
        proposal_constraints: vec!["one-shot".into()],
        choice_set_certificate_hash: String::new(),
        selected_candidate_hash: "candidate-two-task".into(),
        choice_set_candidates: vec!["candidate-two-task".into()],
        selected_candidate_resources: task_robots.clone(),
        planned_start_s: 0.0,
        choice_set_certificate_payload: String::new(),
    };
    let payload = serde_json::json!({
        "schema": "poie.recovery-choice-set-certificate/v2",
        "checkpoint_hash": checkpoint_hash,
        "execution_id": checkpoint.execution_id,
        "feasible_candidates": ["candidate-two-task"],
        "payloads": { "candidate-two-task": { "resources": task_robots, "planned_start_s": 0.0 } }
    })
    .to_string();
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(b"poie.choice-set-certificate/v1\x1f");
    hasher.update(payload.as_bytes());
    let certificate_hash = format!("{:x}", hasher.finalize());
    proposal.choice_set_certificate_hash = certificate_hash.clone();
    proposal.choice_set_certificate_payload = payload;
    poie_viability::engine::register_trusted_choice_set_for_research(&certificate_hash);
    let auth = ReplanAuthorization::issue("n24-auth", &checkpoint, &proposal, "ONE_SHOT_REPLAN");
    let resumed =
        resume_engine_with_authorization_event_observed(&model, &options, &group, &checkpoint, &auth, &proposal)
            .unwrap();
    let to_json = |events: &[poie_viability::engine::EventRecord]| {
        events
            .iter()
            .map(|e| {
                serde_json::json!({
                    "event_id": e.event_id, "sequence": e.event_sequence, "time": e.at_s,
                    "kind": format!("{:?}", e.kind), "target": e.target_asset,
                    "asset_after": e.asset_statuses_after
                })
            })
            .collect::<Vec<_>>()
    };
    let out = serde_json::json!({
        "schema": "poie.repair-trajectory/v1",
        "execution_id": checkpoint.execution_id,
        "group_id": checkpoint.group_id,
        "checkpoint_hash": checkpoint_hash,
        "checkpoint_time": checkpoint.at_s,
        "completed_tasks": checkpoint.completed_tasks, "unfinished_tasks": checkpoint.unfinished_tasks,
        "unfinished_tasks": checkpoint.unfinished_tasks,
        "authorization_id": auth.authorization_id,
        "selected_candidate_hash": proposal.selected_candidate_hash,
        "initial_events": to_json(&initial_events),
        "resume_events": to_json(&resumed.events),
        "canonical_outcome": format!("{:?}", resumed.outcome.termination),
        "survived_s": resumed.outcome.survived_s,
        "checkpoint_asset_statuses": checkpoint.asset_statuses
    });
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}
