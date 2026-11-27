// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later
use poie_ontology::model::{AssetStatus, Model, Service};
use poie_viability::engine::{
    resume_engine_with_authorization, run_engine_with_repair_group_event_observed,
    run_engine_with_repair_group_to_checkpoint, EngineOptions, ExplicitRepairGroup, ExplicitRepairTask,
    RepairCheckpoint, RepairLifecycleMode, ReplanAuthorization, ReplanProposal,
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
    let mut proposal = make_proposal(&cp, "p");
    proposal.task_robots.insert("B".into(), vec!["robot-m2".into()]);
    let auth = unique_auth(&cp, &proposal);
    let resumed = resume_engine_with_authorization(&model, &options, &group(), &cp, &auth, &proposal).unwrap();
    assert_eq!(resumed.termination, poie_viability::engine::EngineTermination::CompletedHorizon);
    println!("CANONICAL_RESULT {{\"completed\":true,\"survived_s\":{},\"failed_robot_m1\":true,\"task_A_rerun\":false,\"inventory_j\":{},\"checkpoint_inventory\":{},\"robot_B\":\"robot-m2\"}}", resumed.survived_s, resumed.final_inventory.get("electric-energy").copied().unwrap_or(0.0), cp.inventory.get("electric-energy").copied().unwrap_or(0.0));
    assert_eq!(resumed.survived_s, 19800.0);
    assert_eq!(cp.asset_statuses.get("robot-m1"), Some(&AssetStatus::Failed));
    assert_eq!(
        resumed.final_inventory.get("electric-energy").copied().unwrap_or(0.0),
        cp.inventory.get("electric-energy").copied().unwrap_or(0.0)
    );
}
fn unique_auth(cp: &RepairCheckpoint, proposal: &ReplanProposal) -> ReplanAuthorization {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    ReplanAuthorization::issue(
        &format!("auth-{}", NEXT.fetch_add(1, Ordering::Relaxed)),
        cp,
        proposal,
        "ONE_SHOT_REPLAN",
    )
}
fn make_proposal(cp: &RepairCheckpoint, id: &str) -> ReplanProposal {
    let mut task_robots = std::collections::BTreeMap::new();
    task_robots.insert("B".to_string(), vec!["robot-m2".to_string()]);
    ReplanProposal {
        proposal_id: id.into(),
        execution_id: cp.execution_id.clone(),
        checkpoint_hash: cp.compute_state_hash(),
        group_id: cp.group_id.clone(),
        unfinished_task_ids: cp.unfinished_tasks.clone(),
        task_robots: task_robots.clone(),
        proposal_constraints: vec!["one-shot".into()],
        choice_set_certificate_hash: "cert".into(),
        selected_candidate_hash: "cand".into(),
        choice_set_candidates: vec!["cand".into()],
        selected_candidate_resources: task_robots,
        planned_start_s: 0.0,
    }
}
fn fixtures() -> (Model, EngineOptions<'static>, ExplicitRepairGroup, RepairCheckpoint, ReplanProposal) {
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
    let (_, checkpoint) = run_engine_with_repair_group_to_checkpoint(&model, &options, &group(), "exec-neg").unwrap();
    let cp = checkpoint.unwrap();
    let mut proposal = make_proposal(&cp, "p");
    proposal.task_robots.insert("B".into(), vec!["robot-m2".into()]);
    (model, options, group(), cp, proposal)
}
#[test]
fn forged_authorization_is_rejected() {
    let (model, options, group, cp, proposal) = fixtures();
    let mut auth = unique_auth(&cp, &proposal);
    auth.checkpoint_hash = "0".repeat(16);
    let out = resume_engine_with_authorization(&model, &options, &group, &cp, &auth, &proposal).unwrap();
    assert!(
        matches!(out.termination, poie_viability::engine::EngineTermination::Blocked(ref r) if r.contains("replan_rejected"))
    );
}
#[test]
fn checkpoint_state_tamper_invalidates_authorization() {
    let (model, options, group, mut cp, proposal) = fixtures();
    let auth = unique_auth(&cp, &proposal);
    cp.asset_statuses.insert("robot-m1".into(), AssetStatus::Available);
    let out = resume_engine_with_authorization(&model, &options, &group, &cp, &auth, &proposal).unwrap();
    assert!(matches!(out.termination, poie_viability::engine::EngineTermination::Blocked(_)));
}
#[test]
fn failed_robot_alternative_is_rejected() {
    let (model, options, group, cp, _) = fixtures();
    let mut proposal = make_proposal(&cp, "p");
    proposal.task_robots.insert("B".into(), vec!["robot-m1".into()]);
    let auth = unique_auth(&cp, &proposal);
    let out = resume_engine_with_authorization(&model, &options, &group, &cp, &auth, &proposal).unwrap();
    assert!(matches!(out.termination, poie_viability::engine::EngineTermination::Blocked(_)));
}
#[test]
fn horizon_insufficient_blocks_continuation() {
    let (model, mut options, group, _, _) = fixtures();
    options.horizon_s = 15000.0;
    let (_, cp) = run_engine_with_repair_group_to_checkpoint(&model, &options, &group, "exec-h").unwrap();
    let cp = cp.unwrap();
    let mut proposal = make_proposal(&cp, "p");
    proposal.task_robots.insert("B".into(), vec!["robot-m2".into()]);
    let auth = unique_auth(&cp, &proposal);
    let out = resume_engine_with_authorization(&model, &options, &group, &cp, &auth, &proposal).unwrap();
    assert!(matches!(
        out.termination,
        poie_viability::engine::EngineTermination::IncompleteAtHorizon(_)
            | poie_viability::engine::EngineTermination::Blocked(_)
    ));
}
#[test]
fn failure_during_continuation_blocks_second_replan() {
    let (model, mut options, group, _, _proposal) = fixtures();
    options.failures.push(FailureEvent { at_s: 15000.0, asset: "robot-m2".into() });
    let (_, cp) = run_engine_with_repair_group_to_checkpoint(&model, &options, &group, "exec-f").unwrap();
    let cp = cp.unwrap();
    let mut proposal = make_proposal(&cp, "p");
    proposal.task_robots.insert("B".into(), vec!["robot-m2".into()]);
    let auth = unique_auth(&cp, &proposal);
    let out = resume_engine_with_authorization(&model, &options, &group, &cp, &auth, &proposal).unwrap();
    assert!(
        matches!(out.termination, poie_viability::engine::EngineTermination::Blocked(ref r) if r.contains("continuation interrupted"))
    );
}

#[test]
fn duplicate_authorization_is_rejected() {
    let (model, options, group, cp, proposal) = fixtures();
    let auth = ReplanAuthorization::issue("dup-auth", &cp, &proposal, "ONE_SHOT_REPLAN");
    let first = resume_engine_with_authorization(&model, &options, &group, &cp, &auth, &proposal).unwrap();
    assert!(matches!(first.termination, poie_viability::engine::EngineTermination::CompletedHorizon));
    let second = resume_engine_with_authorization(&model, &options, &group, &cp, &auth, &proposal).unwrap();
    assert!(
        matches!(second.termination, poie_viability::engine::EngineTermination::Blocked(ref r) if r.contains("already consumed"))
    );
}
#[test]
fn forgotten_failed_asset_with_reissued_authorization_is_rejected() {
    let (model, options, group, mut cp, proposal) = fixtures();
    cp.asset_statuses.insert("robot-m1".into(), AssetStatus::Available);
    cp.state_hash = cp.compute_state_hash();
    let auth = ReplanAuthorization::issue("tamper-auth", &cp, &proposal, "ONE_SHOT_REPLAN");
    let out = resume_engine_with_authorization(&model, &options, &group, &cp, &auth, &proposal).unwrap();
    assert!(
        matches!(out.termination, poie_viability::engine::EngineTermination::Blocked(ref r) if r.contains("lost failure history"))
    );
}
#[test]
fn no_failure_path_creates_no_checkpoint() {
    let model = model();
    let options = EngineOptions {
        services: vec!["svc"],
        horizon_s: 72000.0,
        failures: vec![],
        resupplies: vec![],
        resources: vec!["robot-m1".into(), "robot-m2".into()],
        alternative_pool: vec!["robot-m1".into(), "robot-m2".into()],
        battery: None,
        load_kw: 0.0,
    };
    let (outcome, checkpoint) =
        run_engine_with_repair_group_to_checkpoint(&model, &options, &group(), "exec-nofail").unwrap();
    assert_eq!(outcome.termination, poie_viability::engine::EngineTermination::CompletedHorizon);
    assert!(checkpoint.is_none());
}
#[test]
fn every_recovery_relevant_field_changes_commitment_and_trusted_verifier_rejects_forgery() {
    use poie_viability::engine::{load_checkpoint, save_checkpoint, verify_checkpoint_file, verify_trusted_checkpoint};
    use poie_viability::survival::FailureEvent;
    use std::collections::BTreeSet;
    let (_, _, _, cp, _) = fixtures();
    let original = cp.compute_state_hash();
    let mut mutations: Vec<(&str, RepairCheckpoint)> = Vec::new();
    let mut m = cp.clone();
    m.completed_tasks.push("B".into());
    mutations.push(("completed_tasks", m));
    let mut m = cp.clone();
    m.unfinished_tasks.clear();
    mutations.push(("unfinished_tasks", m));
    let mut m = cp.clone();
    m.selected_resources.insert("A".into(), vec!["robot-m1".into()]);
    mutations.push(("selected_resources", m));
    let mut m = cp.clone();
    m.active_dependencies.push("bus-2".into());
    mutations.push(("active_dependencies", m));
    let mut m = cp.clone();
    m.applied_failures.push(FailureEvent { at_s: 1.0, asset: "fake".into() });
    mutations.push(("applied_failures", m));
    let mut m = cp.clone();
    m.pending_failures.push(FailureEvent { at_s: 1.0, asset: "fake".into() });
    mutations.push(("pending_failures", m));
    let mut m = cp.clone();
    m.remaining_failures.push(FailureEvent { at_s: 1.0, asset: "fake".into() });
    mutations.push(("remaining_failures", m));
    let mut m = cp.clone();
    m.inventory.insert("fake".into(), 1.0);
    mutations.push(("inventory", m));
    let mut m = cp.clone();
    m.asset_statuses.insert("robot-m2".into(), AssetStatus::Degraded);
    mutations.push(("asset_statuses", m));
    let mut m = cp.clone();
    m.completed_plans[0].total_duration_s += 1.0;
    mutations.push(("completed_plans", m));
    let mut m = cp.clone();
    m.completed_planning_base.insert("fake".into(), 1.0);
    mutations.push(("completed_planning_base", m));
    let mut m = cp.clone();
    m.trace_history[0].kind = "mutated".into();
    mutations.push(("trace_history", m));
    let mut m = cp.clone();
    m.predecessor_hash = "forged".into();
    mutations.push(("predecessor_hash", m));
    let mut m = cp.clone();
    m.event_sequence += 1;
    mutations.push(("event_sequence", m));
    let mut m = cp.clone();
    m.event_batch_sequence += 1;
    mutations.push(("event_batch_sequence", m));
    let mut m = cp.clone();
    m.source_environment = "forged".into();
    mutations.push(("source_environment", m));
    for (name, mutated) in mutations {
        assert_ne!(mutated.compute_state_hash(), original, "field omission: {name}");
    }
    let trusted: BTreeSet<String> = BTreeSet::from([original.clone()]);
    assert!(verify_trusted_checkpoint(&cp, &trusted).is_ok());
    let mut forged = cp.clone();
    forged.inventory.insert("fake".into(), 1.0);
    forged.state_hash = forged.compute_state_hash();
    assert!(verify_trusted_checkpoint(&forged, &trusted).is_err(), "recomputed untrusted hash accepted");
    let path = std::env::temp_dir().join(format!("poie-n21rh-{}.ckpt", std::process::id()));
    save_checkpoint(&path, &cp).unwrap();
    assert!(verify_checkpoint_file(&path, &trusted).is_ok());
    let loaded = load_checkpoint(&path, &cp.execution_id, &trusted).unwrap();
    assert_eq!(loaded.compute_state_hash(), cp.compute_state_hash());
    assert!(load_checkpoint(&path, "wrong-exec", &trusted).is_err());
    std::fs::write(&path, b"tampered").unwrap();
    assert!(verify_checkpoint_file(&path, &trusted).is_err());
    std::fs::remove_file(&path).ok();
}
#[test]
fn durable_ledger_rejects_crash_before_commit_and_second_claim() {
    use poie_viability::engine::{FileAuthorizationLedger, LedgerClaim};
    let dir = std::env::temp_dir().join(format!("poie-n21rh-ledger-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let ledger = FileAuthorizationLedger::new(&dir);
    assert_eq!(ledger.claim_id("auth-crash").unwrap(), LedgerClaim::Claimed);
    assert_eq!(ledger.claim_id("auth-crash").unwrap(), LedgerClaim::InProgressOrFailedClosed);
    ledger.commit_id("auth-crash", "result-1").unwrap();
    assert_eq!(ledger.claim_id("auth-crash").unwrap(), LedgerClaim::AlreadyCommitted);
    std::fs::remove_dir_all(&dir).ok();
}
#[test]
fn cross_process_concurrent_claim_has_exactly_one_winner() {
    use std::process::{Command, Stdio};
    let dir = std::env::temp_dir().join(format!("poie-n21rh-xproc-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let exe = env!("CARGO_BIN_EXE_poie-s5v-n21rh-ledger-claim");
    let mut children: Vec<_> = (0..8)
        .map(|_| {
            Command::new(exe).arg(&dir).arg("same-auth").stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap()
        })
        .collect();
    let mut winners = 0;
    for child in &mut children {
        if child.wait().unwrap().success() {
            winners += 1;
        }
    }
    assert_eq!(winners, 1, "exactly one process may claim the authorization");
    std::fs::remove_dir_all(&dir).ok();
}
#[test]
fn authorization_signature_issuer_expiry_and_proposal_binding_are_enforced() {
    let (model, options, group, cp, proposal) = fixtures();
    let mut auth = unique_auth(&cp, &proposal);
    auth.issuer_id = "forged-issuer".into();
    assert!(matches!(
        resume_engine_with_authorization(&model, &options, &group, &cp, &auth, &proposal).unwrap().termination,
        poie_viability::engine::EngineTermination::Blocked(_)
    ));
    let mut auth = unique_auth(&cp, &proposal);
    auth.signature = "00".into();
    assert!(matches!(
        resume_engine_with_authorization(&model, &options, &group, &cp, &auth, &proposal).unwrap().termination,
        poie_viability::engine::EngineTermination::Blocked(_)
    ));
    let mut auth = unique_auth(&cp, &proposal);
    auth.expiration_s = 0.0;
    auth.signature = String::new();
    assert!(auth.verify(&cp, &proposal, cp.at_s).is_err());
    let auth = unique_auth(&cp, &proposal);
    let mut other = proposal.clone();
    other.proposal_id = "other".into();
    assert!(auth.verify(&cp, &other, cp.at_s).is_err());
}
#[test]
fn durable_resume_commits_and_blocks_replay() {
    use poie_viability::engine::{resume_engine_with_durable_authorization, FileAuthorizationLedger};
    let (model, options, group, cp, proposal) = fixtures();
    let auth = unique_auth(&cp, &proposal);
    let dir = std::env::temp_dir().join(format!("poie-n21rh-durable-resume-{}", std::process::id()));
    let ledger = FileAuthorizationLedger::new(&dir);
    let first =
        resume_engine_with_durable_authorization(&model, &options, &group, &cp, &auth, &proposal, &ledger).unwrap();
    assert_eq!(first.termination, poie_viability::engine::EngineTermination::CompletedHorizon);
    let second =
        resume_engine_with_durable_authorization(&model, &options, &group, &cp, &auth, &proposal, &ledger).unwrap();
    assert!(matches!(second.termination, poie_viability::engine::EngineTermination::Blocked(_)));
    std::fs::remove_dir_all(&dir).ok();
}
#[test]
fn proposal_mutations_after_authorization_are_rejected() {
    let (model, options, group, cp, _) = fixtures();
    let mut proposal = make_proposal(&cp, "proposal-v13");
    proposal.task_robots.insert("B".into(), vec!["robot-m2".into()]);
    let auth = unique_auth(&cp, &proposal);
    let mut changed = proposal.clone();
    changed.selected_candidate_hash = "other-candidate".into();
    assert!(matches!(
        resume_engine_with_authorization(&model, &options, &group, &cp, &auth, &changed).unwrap().termination,
        poie_viability::engine::EngineTermination::Blocked(_)
    ));
    let mut changed = proposal.clone();
    changed.checkpoint_hash = "0".repeat(64);
    assert!(matches!(
        resume_engine_with_authorization(&model, &options, &group, &cp, &auth, &changed).unwrap().termination,
        poie_viability::engine::EngineTermination::Blocked(_)
    ));
    let mut changed = proposal.clone();
    changed.execution_id = "other-exec".into();
    assert!(matches!(
        resume_engine_with_authorization(&model, &options, &group, &cp, &auth, &changed).unwrap().termination,
        poie_viability::engine::EngineTermination::Blocked(_)
    ));
    let valid = resume_engine_with_authorization(&model, &options, &group, &cp, &auth, &proposal).unwrap();
    assert_eq!(valid.termination, poie_viability::engine::EngineTermination::CompletedHorizon);
}

#[test]
fn native_execution_requires_authorized_candidate_membership() {
    let (model, mut options, group, cp, _) = fixtures();
    options.resources.push("robot-m3".into());
    options.alternative_pool.push("robot-m3".into());
    let mut proposal = make_proposal(&cp, "proposal-m3");
    proposal.task_robots.insert("B".into(), vec!["robot-m3".into()]);
    proposal.selected_candidate_resources.insert("B".into(), vec!["robot-m3".into()]);
    proposal.selected_candidate_hash = "candidate-m3".into();
    proposal.choice_set_candidates = vec!["candidate-m2".into(), "candidate-m3".into()];
    let auth = unique_auth(&cp, &proposal);
    let ok = resume_engine_with_authorization(&model, &options, &group, &cp, &auth, &proposal).unwrap();
    assert_eq!(ok.termination, poie_viability::engine::EngineTermination::CompletedHorizon);

    let mut missing = proposal.clone();
    missing.selected_candidate_hash = "candidate-m9".into();
    let missing_auth = unique_auth(&cp, &missing);
    let blocked = resume_engine_with_authorization(&model, &options, &group, &cp, &missing_auth, &missing).unwrap();
    assert!(
        matches!(blocked.termination, poie_viability::engine::EngineTermination::Blocked(ref r) if r.contains("not in frozen choice set"))
    );

    let mut mismatch = proposal.clone();
    mismatch.selected_candidate_resources.insert("B".into(), vec!["robot-m2".into()]);
    let mismatch_auth = unique_auth(&cp, &mismatch);
    let blocked = resume_engine_with_authorization(&model, &options, &group, &cp, &mismatch_auth, &mismatch).unwrap();
    assert!(
        matches!(blocked.termination, poie_viability::engine::EngineTermination::Blocked(ref r) if r.contains("resources do not match"))
    );
}
#[test]
fn n23_repro_self_attested_choice_set_is_accepted() {
    let (model, mut options, group, cp, _) = fixtures();
    options.resources.push("robot-m3".into());
    options.alternative_pool.push("robot-m3".into());
    let mut proposal = make_proposal(&cp, "self-attested");
    proposal.choice_set_certificate_hash = "fabricated-certificate".into();
    proposal.selected_candidate_hash = "fabricated-candidate".into();
    proposal.choice_set_candidates = vec!["fabricated-candidate".into()];
    proposal.task_robots.insert("B".into(), vec!["robot-m3".into()]);
    proposal.selected_candidate_resources.insert("B".into(), vec!["robot-m3".into()]);
    let auth = unique_auth(&cp, &proposal);
    let outcome = resume_engine_with_authorization(&model, &options, &group, &cp, &auth, &proposal).unwrap();
    println!("N23_SELF_ATTESTED_EVIDENCE {:?}", outcome.termination);
    assert_eq!(outcome.termination, poie_viability::engine::EngineTermination::CompletedHorizon);
}
#[test]
fn canonical_committed_planned_start_is_enforced() {
    let (model, mut options, group, cp, _) = fixtures();
    options.resources.push("robot-m3".into());
    options.alternative_pool.push("robot-m3".into());
    let mut proposal = make_proposal(&cp, "planned-start");
    proposal.task_robots.insert("B".into(), vec!["robot-m3".into()]);
    proposal.selected_candidate_resources.insert("B".into(), vec!["robot-m3".into()]);
    proposal.planned_start_s = 10000.0;
    let auth = unique_auth(&cp, &proposal);
    let outcome = resume_engine_with_authorization(&model, &options, &group, &cp, &auth, &proposal).unwrap();
    assert_eq!(outcome.termination, poie_viability::engine::EngineTermination::CompletedHorizon);
    assert!((outcome.survived_s - 20800.0).abs() < 1e-9, "committed completion {}", outcome.survived_s);

    let mut legacy = make_proposal(&cp, "legacy");
    legacy.planned_start_s = 0.0;
    let auth = unique_auth(&cp, &legacy);
    let outcome = resume_engine_with_authorization(&model, &options, &group, &cp, &auth, &legacy).unwrap();
    assert!((outcome.survived_s - 19800.0).abs() < 1e-9);
}
