// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later
use poie_ontology::model::{AssetStatus, Model, Service};
use poie_viability::engine::{
    run_engine_with_repair_group_to_checkpoint, verify_trusted_checkpoint, EngineOptions, ExplicitRepairGroup,
    ExplicitRepairTask, RepairLifecycleMode,
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
    let (_, checkpoint) = run_engine_with_repair_group_to_checkpoint(&model, &options, &group(), "n23-exec").unwrap();
    let checkpoint = checkpoint.expect("interruption checkpoint");
    let mut trusted = BTreeSet::new();
    trusted.insert(checkpoint.compute_state_hash());
    verify_trusted_checkpoint(&checkpoint, &trusted).unwrap();
    let failed: Vec<String> = checkpoint
        .asset_statuses
        .iter()
        .filter(|(_, s)| **s == AssetStatus::Failed)
        .map(|(id, _)| id.clone())
        .collect();
    let available: Vec<String> = checkpoint
        .asset_statuses
        .iter()
        .filter(|(_, s)| **s == AssetStatus::Available)
        .map(|(id, _)| id.clone())
        .collect();
    let projection = serde_json::json!({
        "schema": "poie.s5v-n23-checkpoint-projection/1",
        "execution_id": checkpoint.execution_id,
        "group_id": checkpoint.group_id,
        "checkpoint_sequence": checkpoint.checkpoint_sequence,
        "checkpoint_commitment": checkpoint.compute_state_hash(),
        "checkpoint_time": checkpoint.at_s,
        "completed_tasks": checkpoint.completed_tasks,
        "unfinished_tasks": checkpoint.unfinished_tasks,
        "failed_assets": failed,
        "available_assets": available,
        "inventory": checkpoint.inventory,
        "remaining_horizon": checkpoint.remaining_horizon_s,
        "selected_resources": checkpoint.selected_resources,
        "active_dependencies": checkpoint.active_dependencies,
        "restoration_target": checkpoint.restoration_target,
        "source_environment": checkpoint.source_environment,
        "trusted_provenance_verified": true
    });
    println!("{}", serde_json::to_string_pretty(&projection).unwrap());
}
