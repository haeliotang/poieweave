// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Canonical event execution kernel.
//!
//! One clock, one inventory ledger and one commit rule for initial repair,
//! failures, resupply and horizon. The kernel emits a typed termination, a
//! trace with before/after inventories, and the plan processes it applied.
//! Legacy `survival` entry points are not yet migrated; this is the target
//! implementation for S5-T.

use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

use poie_ontology::model::{Asset, AssetStatus, Model, Service};

use crate::clock::EventClock;
use crate::closure::{analyze, ClosureOutcome, Plan, Step, StepAction};
use crate::scheduler::{
    schedule_plans, schedule_plans_with_alternatives, schedule_tasks, ResourceRequirement, SchedulableTask,
};
use crate::survival::{
    apply_energy_interval_ledger, apply_plans, energy, initial_inventory, model_with_inventory, BatteryLimits,
    EnergyFlow, FailureEvent, Resupply,
};

/// Explicit terminal state; never inferred from log text.
#[derive(Clone, Debug, PartialEq)]
pub enum EngineTermination {
    CompletedHorizon,
    Blocked(String),
    IncompleteAtHorizon(String),
}

/// One capability choice and the assets it depends on during the repair.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WitnessEntry {
    pub capability: String,
    pub provider: String,
    pub dependencies: Vec<String>,
}

/// One committed event.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EngineTrace {
    pub at_s: f64,
    pub kind: String,
    pub inventory_before: BTreeMap<String, f64>,
    pub inventory_after: BTreeMap<String, f64>,
    pub plans: Vec<String>,
    pub resources: Vec<String>,
    pub witness: Vec<WitnessEntry>,
    pub energy_generated_j: f64,
    pub energy_resupply_j: f64,
    pub energy_load_j: f64,
    pub energy_repair_j: f64,
    pub energy_curtailed_j: f64,
}

/// Full canonical result.
#[derive(Clone, Debug, PartialEq)]
pub struct EngineOutcome {
    pub termination: EngineTermination,
    pub survived_s: f64,
    pub trace: Vec<EngineTrace>,
    pub final_inventory: BTreeMap<String, f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObservationPhase {
    Before,
    After,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObservationRequest {
    pub at_s: f64,
    pub phase: ObservationPhase,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RepairObservation {
    pub label: String,
    pub end_s: f64,
    pub plans: Vec<String>,
    pub resources: Vec<String>,
    pub execution_dependencies: Vec<String>,
    pub restoration_targets: Vec<String>,
    pub witness: Vec<WitnessEntry>,
    pub task_resources: Vec<TaskResourceObservation>,
    pub completed_tasks: Vec<String>,
    pub active_resources: Vec<String>,
    pub future_resources: Vec<String>,
    pub released_resources: Vec<String>,
}

/// Observation-only per-plan resource assignment derived from the actual schedule.
#[derive(Clone, Debug, PartialEq)]
pub struct TaskResourceObservation {
    pub plan_index: usize,
    pub task_id: Option<String>,
    pub resources: Vec<String>,
    pub start_s: f64,
    pub end_s: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservationStatus {
    Observed,
    UnobservableAfterTermination,
    UnobservableNotMaterialized,
}

impl ObservationStatus {
    pub fn label(self) -> &'static str {
        match self {
            ObservationStatus::Observed => "OBSERVED",
            ObservationStatus::UnobservableAfterTermination => "UNOBSERVABLE_AFTER_TERMINATION",
            ObservationStatus::UnobservableNotMaterialized => "UNOBSERVABLE_NOT_MATERIALIZED",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EngineObservation {
    pub at_s: f64,
    pub phase: ObservationPhase,
    pub status: ObservationStatus,
    pub inventory: BTreeMap<String, f64>,
    pub energy_generated_j: f64,
    pub energy_load_j: f64,
    pub energy_repair_j: f64,
    pub energy_curtailed_j: f64,
    pub energy_resupply_j: f64,
    pub running_repair: Option<RepairObservation>,
    /// Native snapshot of the canonical runtime model at the observation boundary.
    pub asset_statuses: BTreeMap<String, AssetStatus>,
    pub asset_status_source: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventKind {
    InitialRepairStarted,
    RepairStarted,
    RepairCompleted,
    AssetFailure,
    Resupply,
    RepairInterrupted,
    EnergyDepleted,
    HorizonReached,
    TaskCompleted,
    ResourceReleased,
    CheckpointCreated,
    ReplanAuthorized,
    ReplanRejected,
    RepairResumed,
    TaskRestarted,
    RestorationCommitted,
}

impl EventKind {
    pub fn label(self) -> &'static str {
        match self {
            EventKind::InitialRepairStarted => "InitialRepairStarted",
            EventKind::RepairStarted => "RepairStarted",
            EventKind::RepairCompleted => "RepairCompleted",
            EventKind::AssetFailure => "AssetFailure",
            EventKind::Resupply => "Resupply",
            EventKind::RepairInterrupted => "RepairInterrupted",
            EventKind::EnergyDepleted => "EnergyDepleted",
            EventKind::HorizonReached => "HorizonReached",
            EventKind::TaskCompleted => "TaskCompleted",
            EventKind::ResourceReleased => "ResourceReleased",
            EventKind::CheckpointCreated => "CheckpointCreated",
            EventKind::ReplanAuthorized => "ReplanAuthorized",
            EventKind::ReplanRejected => "ReplanRejected",
            EventKind::RepairResumed => "RepairResumed",
            EventKind::TaskRestarted => "TaskRestarted",
            EventKind::RestorationCommitted => "RestorationCommitted",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventOrigin {
    Exogenous,
    Endogenous,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventStatus {
    Committed,
    Rejected,
    NotApplied,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EventRecord {
    pub event_sequence: usize,
    pub event_id: String,
    pub event_batch_sequence: usize,
    pub at_s: f64,
    pub kind: EventKind,
    pub origin: EventOrigin,
    pub target_asset: Option<String>,
    pub status: EventStatus,
    pub asset_statuses_before: BTreeMap<String, AssetStatus>,
    pub asset_statuses_after: BTreeMap<String, AssetStatus>,
    pub inventory_before: BTreeMap<String, f64>,
    pub inventory_after: BTreeMap<String, f64>,
    pub running_repair: Option<RepairObservation>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EventObservedOutcome {
    pub outcome: EngineOutcome,
    pub events: Vec<EventRecord>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ObservedEngineOutcome {
    pub outcome: EngineOutcome,
    pub observations: Vec<EngineObservation>,
}

#[derive(Default)]
struct ObservationCollector {
    requests: Vec<ObservationRequest>,
    index: usize,
    observations: Vec<EngineObservation>,
}

thread_local! {
    static OBSERVATION_COLLECTOR: std::cell::RefCell<Option<ObservationCollector>> =
        const { std::cell::RefCell::new(None) };
}

struct ObservationGuard {
    previous: Option<ObservationCollector>,
}

impl ObservationGuard {
    fn install(collector: ObservationCollector) -> Result<Self, EngineError> {
        OBSERVATION_COLLECTOR.with(|cell| {
            let mut slot = cell.borrow_mut();
            if slot.is_some() {
                return Err(EngineError("nested run_engine_observed is not supported".into()));
            }
            let previous = slot.take();
            *slot = Some(collector);
            Ok(ObservationGuard { previous })
        })
    }
}

impl Drop for ObservationGuard {
    fn drop(&mut self) {
        OBSERVATION_COLLECTOR.with(|cell| {
            *cell.borrow_mut() = self.previous.take();
        });
    }
}

thread_local! {
    static EVENT_COLLECTOR: std::cell::RefCell<Option<Vec<EventRecord>>> =
        const { std::cell::RefCell::new(None) };
}

struct EventGuard {
    previous: Option<Vec<EventRecord>>,
}

impl EventGuard {
    fn install() -> Result<Self, EngineError> {
        EVENT_COLLECTOR.with(|cell| {
            let mut slot = cell.borrow_mut();
            if slot.is_some() {
                return Err(EngineError("nested event observation is not supported".into()));
            }
            let previous = slot.take();
            *slot = Some(Vec::new());
            Ok(EventGuard { previous })
        })
    }
}

impl Drop for EventGuard {
    fn drop(&mut self) {
        EVENT_COLLECTOR.with(|cell| {
            *cell.borrow_mut() = self.previous.take();
        });
    }
}

fn event_active() -> bool {
    EVENT_COLLECTOR.with(|cell| cell.borrow().is_some())
}

#[allow(clippy::too_many_arguments)]
fn repair_observation(repair: &RunningRepair, at_s: f64) -> RepairObservation {
    let mut completed = Vec::new();
    let mut active = BTreeSet::new();
    let mut future = BTreeSet::new();
    for (index, task) in repair.task_resources.iter().enumerate() {
        if repair.completed_tasks.contains(&index) {
            completed.push(task.task_id.clone().unwrap_or_else(|| format!("task-{index}")));
        } else if task.start_s <= at_s + 1e-9 && at_s < task.end_s - 1e-9 {
            active.extend(task.resources.iter().cloned());
        } else if at_s < task.start_s - 1e-9 {
            future.extend(task.resources.iter().cloned());
        }
    }
    RepairObservation {
        label: repair.label.clone(),
        end_s: repair.end_s,
        plans: plan_summary(&repair.plans),
        resources: repair.resources.clone(),
        execution_dependencies: repair.execution_dependencies.iter().cloned().collect(),
        restoration_targets: repair.restoration_targets.iter().cloned().collect(),
        witness: repair.witness.clone(),
        task_resources: repair.task_resources.clone(),
        completed_tasks: completed,
        active_resources: active.into_iter().collect(),
        future_resources: future.into_iter().collect(),
        released_resources: repair.released_resources.iter().cloned().collect(),
    }
}

#[allow(clippy::too_many_arguments)]
fn record_event(
    sequence: usize,
    batch: usize,
    at_s: f64,
    kind: EventKind,
    origin: EventOrigin,
    target: Option<String>,
    status: EventStatus,
    before: &BTreeMap<String, AssetStatus>,
    after: &BTreeMap<String, AssetStatus>,
    inventory_before: &BTreeMap<String, f64>,
    inventory_after: &BTreeMap<String, f64>,
    running: Option<&RunningRepair>,
) {
    if !event_active() {
        return;
    }
    let event = EventRecord {
        event_sequence: sequence,
        event_id: format!("ev-{sequence:04}-{}", kind.label()),
        event_batch_sequence: batch,
        at_s,
        kind,
        origin,
        target_asset: target,
        status,
        asset_statuses_before: before.clone(),
        asset_statuses_after: after.clone(),
        inventory_before: inventory_before.clone(),
        inventory_after: inventory_after.clone(),
        running_repair: running.map(|repair| repair_observation(repair, at_s)),
    };
    EVENT_COLLECTOR.with(|cell| {
        if let Some(events) = cell.borrow_mut().as_mut() {
            events.push(event);
        }
    });
}

/// Runs the kernel while recording every committed event and its native
/// before/after asset status snapshot.
pub fn run_engine_event_observed(
    model: &Model,
    options: &EngineOptions<'_>,
) -> Result<EventObservedOutcome, EngineError> {
    let guard = EventGuard::install()?;
    let outcome = run_engine(model, options);
    let events = EVENT_COLLECTOR.with(|cell| cell.borrow_mut().take()).unwrap_or_default();
    drop(guard);
    outcome.map(|outcome| EventObservedOutcome { outcome, events })
}

fn with_observations(mut f: impl FnMut(&mut ObservationCollector)) {
    OBSERVATION_COLLECTOR.with(|cell| {
        if let Some(collector) = cell.borrow_mut().as_mut() {
            f(collector);
        }
    });
}

fn next_request_time() -> Option<f64> {
    let mut value = None;
    with_observations(|collector| {
        value = collector.requests.get(collector.index).map(|request| request.at_s);
    });
    value
}

fn asset_status_snapshot(assets: &BTreeMap<String, Asset>) -> BTreeMap<String, AssetStatus> {
    assets.iter().map(|(id, asset)| (id.clone(), asset.status)).collect()
}

fn observe_due(
    at_s: f64,
    phase: ObservationPhase,
    inventory: &BTreeMap<String, f64>,
    running: Option<&RunningRepair>,
    trace: &[EngineTrace],
    interval: &EnergyFlow,
    assets: &BTreeMap<String, AssetStatus>,
) {
    with_observations(|collector| {
        while let Some(request) = collector.requests.get(collector.index).copied() {
            if (request.at_s - at_s).abs() > 1e-6 {
                break;
            }
            if request.phase != phase {
                break;
            }
            let (generated, load, repair, curtailed, resupply) = trace.iter().fold(
                (interval.generated_j, interval.load_j, 0.0, interval.curtailed_j, 0.0),
                |mut acc, entry| {
                    acc.0 += entry.energy_generated_j;
                    acc.1 += entry.energy_load_j;
                    acc.2 += entry.energy_repair_j;
                    acc.3 += entry.energy_curtailed_j;
                    acc.4 += entry.energy_resupply_j;
                    acc
                },
            );
            collector.observations.push(EngineObservation {
                at_s,
                phase,
                status: ObservationStatus::Observed,
                inventory: inventory.clone(),
                energy_generated_j: generated,
                energy_load_j: load,
                energy_repair_j: repair,
                energy_curtailed_j: curtailed,
                energy_resupply_j: resupply,
                running_repair: running.map(|repair| repair_observation(repair, at_s)),
                asset_statuses: assets.clone(),
                asset_status_source: "NATIVE_CANONICAL_RUNTIME",
            });
            collector.index += 1;
        }
    });
}

/// Runs the canonical kernel with native, read-only observations.
pub fn run_engine_observed(
    model: &Model,
    options: &EngineOptions<'_>,
    requests: &[ObservationRequest],
) -> Result<ObservedEngineOutcome, EngineError> {
    let mut sorted = requests.to_vec();
    sorted.sort_by(|left, right| left.at_s.total_cmp(&right.at_s).then_with(|| left.phase.cmp(&right.phase)));
    let collector = ObservationCollector { requests: sorted.clone(), index: 0, observations: Vec::new() };
    let guard = ObservationGuard::install(collector)?;
    let outcome = run_engine(model, options);
    let collected = OBSERVATION_COLLECTOR.with(|cell| cell.borrow_mut().take());
    drop(guard);
    let result = outcome.map(|outcome| {
        let mut observations = collected.map(|collector| collector.observations).unwrap_or_default();
        for request in sorted {
            if !observations.iter().any(|observation| {
                (observation.at_s - request.at_s).abs() <= 1e-6 && observation.phase == request.phase
            }) {
                observations.push(EngineObservation {
                    at_s: request.at_s,
                    phase: request.phase,
                    status: ObservationStatus::UnobservableAfterTermination,
                    inventory: BTreeMap::new(),
                    energy_generated_j: 0.0,
                    energy_load_j: 0.0,
                    energy_repair_j: 0.0,
                    energy_curtailed_j: 0.0,
                    energy_resupply_j: 0.0,
                    running_repair: None,
                    asset_statuses: BTreeMap::new(),
                    asset_status_source: "UNOBSERVABLE",
                });
            }
        }
        observations.sort_by(|left, right| left.at_s.total_cmp(&right.at_s).then_with(|| left.phase.cmp(&right.phase)));
        ObservedEngineOutcome { outcome, observations }
    });
    result
}

#[derive(Clone, Debug, PartialEq)]
pub struct EngineError(pub String);

/// Inputs for one canonical run.
#[derive(Clone, Debug)]
pub struct EngineOptions<'a> {
    pub services: Vec<&'a str>,
    pub horizon_s: f64,
    pub failures: Vec<FailureEvent>,
    pub resupplies: Vec<Resupply>,
    pub resources: Vec<String>,
    pub alternative_pool: Vec<String>,
    pub battery: Option<BatteryLimits>,
    pub load_kw: f64,
}

/// Native v11 checkpoint captured at a repair interruption.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RepairCheckpoint {
    pub execution_id: String,
    pub group_id: String,
    pub checkpoint_sequence: u64,
    pub at_s: f64,
    pub asset_statuses: BTreeMap<String, AssetStatus>,
    pub inventory: BTreeMap<String, f64>,
    pub completed_tasks: Vec<String>,
    pub unfinished_tasks: Vec<String>,
    pub selected_resources: BTreeMap<String, Vec<String>>,
    pub active_dependencies: Vec<String>,
    pub applied_failures: Vec<FailureEvent>,
    pub pending_failures: Vec<FailureEvent>,
    pub remaining_failures: Vec<FailureEvent>,
    pub group_status: String,
    pub restoration_target: String,
    pub event_sequence: usize,
    pub event_batch_sequence: usize,
    pub remaining_horizon_s: f64,
    pub trace_history: Vec<EngineTrace>,
    pub completed_plans: Vec<Plan>,
    pub completed_planning_base: BTreeMap<String, f64>,
    pub source_environment: String,
    pub state_hash: String,
    pub predecessor_hash: String,
}

impl RepairCheckpoint {
    pub fn compute_state_hash(&self) -> String {
        let bytes = self.canonical_state_bytes().expect("checkpoint contains non-finite float");
        let mut hasher = Sha256::new();
        hasher.update(b"poie.repair-checkpoint/v12\x1f");
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    }

    /// Deterministic canonical encoding of every recovery-relevant field.
    pub fn canonical_state_bytes(&self) -> Result<Vec<u8>, String> {
        fn f(x: f64) -> Result<String, String> {
            if !x.is_finite() {
                return Err("non-finite float in checkpoint".into());
            }
            let x = if x == 0.0 { 0.0 } else { x };
            Ok(format!("{:.17e}", x))
        }
        fn s(out: &mut String, tag: &str, value: &str) {
            out.push_str(&format!("|{tag}:{}:{}", value.len(), value));
        }
        let mut out = String::from("poie.repair-checkpoint/v12");
        s(&mut out, "execution_id", &self.execution_id);
        s(&mut out, "group_id", &self.group_id);
        s(&mut out, "sequence", &self.checkpoint_sequence.to_string());
        s(&mut out, "at_s", &f(self.at_s)?);
        for (asset, status) in &self.asset_statuses {
            s(&mut out, "asset", &format!("{asset}:{status:?}"));
        }
        for (resource, amount) in &self.inventory {
            s(&mut out, "inventory", &format!("{resource}:{}", f(*amount)?));
        }
        for task in &self.completed_tasks {
            s(&mut out, "completed_task", task);
        }
        for task in &self.unfinished_tasks {
            s(&mut out, "unfinished_task", task);
        }
        for (task, resources) in &self.selected_resources {
            s(&mut out, "selected", &format!("{task}:{}", resources.join(",")));
        }
        for dep in &self.active_dependencies {
            s(&mut out, "active_dependency", dep);
        }
        let mut failures = |tag: &str, list: &[FailureEvent]| -> Result<(), String> {
            s(&mut out, &format!("{tag}_count"), &list.len().to_string());
            for item in list {
                s(&mut out, tag, &format!("{}:{}", f(item.at_s)?, item.asset));
            }
            Ok(())
        };
        failures("applied_failure", &self.applied_failures)?;
        failures("pending_failure", &self.pending_failures)?;
        failures("remaining_failure", &self.remaining_failures)?;
        s(&mut out, "group_status", &self.group_status);
        s(&mut out, "restoration_target", &self.restoration_target);
        s(&mut out, "event_sequence", &self.event_sequence.to_string());
        s(&mut out, "event_batch_sequence", &self.event_batch_sequence.to_string());
        s(&mut out, "remaining_horizon", &f(self.remaining_horizon_s)?);
        for trace in &self.trace_history {
            s(&mut out, "trace_at", &f(trace.at_s)?);
            s(&mut out, "trace_kind", &trace.kind);
            for (k, v) in &trace.inventory_before {
                s(&mut out, "trace_before", &format!("{k}:{}", f(*v)?));
            }
            for (k, v) in &trace.inventory_after {
                s(&mut out, "trace_after", &format!("{k}:{}", f(*v)?));
            }
            for plan in &trace.plans {
                s(&mut out, "trace_plan", plan);
            }
            for resource in &trace.resources {
                s(&mut out, "trace_resource", resource);
            }
            for entry in &trace.witness {
                s(
                    &mut out,
                    "witness",
                    &format!("{}:{}:{}", entry.capability, entry.provider, entry.dependencies.join(",")),
                );
            }
            s(
                &mut out,
                "trace_energy",
                &format!(
                    "{}:{}:{}:{}:{}",
                    f(trace.energy_generated_j)?,
                    f(trace.energy_resupply_j)?,
                    f(trace.energy_load_j)?,
                    f(trace.energy_repair_j)?,
                    f(trace.energy_curtailed_j)?
                ),
            );
        }
        for plan in &self.completed_plans {
            s(&mut out, "completed_plan", &serde_json::to_string(plan).map_err(|e| e.to_string())?);
        }
        for (k, v) in &self.completed_planning_base {
            s(&mut out, "planning_base", &format!("{k}:{}", f(*v)?));
        }
        s(&mut out, "predecessor_hash", &self.predecessor_hash);
        s(&mut out, "source_environment", &self.source_environment);
        Ok(out.into_bytes())
    }
}

/// Verify that a checkpoint commitment appears in a trusted provenance ledger.
/// A modified checkpoint with a recomputed hash is rejected unless its new
/// commitment was independently admitted to the ledger.
pub fn verify_trusted_checkpoint(checkpoint: &RepairCheckpoint, trusted: &BTreeSet<String>) -> Result<(), String> {
    let commitment = checkpoint.compute_state_hash();
    if trusted.contains(&commitment) {
        Ok(())
    } else {
        Err("checkpoint not in trusted provenance ledger".into())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CheckpointFile {
    schema: String,
    checkpoint: RepairCheckpoint,
}

/// Atomically persist a checkpoint as a versioned JSON envelope.
pub fn save_checkpoint(path: &std::path::Path, checkpoint: &RepairCheckpoint) -> Result<(), String> {
    checkpoint.canonical_state_bytes()?;
    let envelope = CheckpointFile { schema: "poie.repair-checkpoint-file/v12".into(), checkpoint: checkpoint.clone() };
    let bytes = serde_json::to_vec(&envelope).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

/// Load a checkpoint, enforcing schema, identity, complete commitment and trusted provenance.
pub fn load_checkpoint(
    path: &std::path::Path,
    expected_execution_id: &str,
    trusted: &BTreeSet<String>,
) -> Result<RepairCheckpoint, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let envelope: CheckpointFile = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if envelope.schema != "poie.repair-checkpoint-file/v12" {
        return Err("checkpoint schema mismatch".into());
    }
    let checkpoint = envelope.checkpoint;
    if checkpoint.execution_id != expected_execution_id {
        return Err("checkpoint execution identity mismatch".into());
    }
    if checkpoint.state_hash != checkpoint.compute_state_hash() {
        return Err("checkpoint commitment mismatch".into());
    }
    if !trusted.contains(&checkpoint.state_hash) {
        return Err("checkpoint not in trusted provenance ledger".into());
    }
    Ok(checkpoint)
}

/// Verify that a persisted checkpoint file is loadable and trusted.
pub fn verify_checkpoint_file(path: &std::path::Path, trusted: &BTreeSet<String>) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let envelope: CheckpointFile = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let checkpoint = envelope.checkpoint;
    if envelope.schema != "poie.repair-checkpoint-file/v12" {
        return Err("checkpoint schema mismatch".into());
    }
    if checkpoint.state_hash != checkpoint.compute_state_hash() {
        return Err("checkpoint commitment mismatch".into());
    }
    if !trusted.contains(&checkpoint.state_hash) {
        return Err("checkpoint file not in trusted provenance ledger".into());
    }
    Ok(())
}

/// One bounded replan proposal: replacement robots for unfinished tasks.
#[derive(Clone, Debug, PartialEq)]
pub struct ReplanProposal {
    pub proposal_id: String,
    pub execution_id: String,
    pub checkpoint_hash: String,
    pub group_id: String,
    pub unfinished_task_ids: Vec<String>,
    pub task_robots: BTreeMap<String, Vec<String>>,
    pub proposal_constraints: Vec<String>,
    pub choice_set_certificate_hash: String,
    pub selected_candidate_hash: String,
}

impl ReplanProposal {
    pub fn compute_hash(&self) -> String {
        let mut text = String::from("poie.replan-proposal/v13");
        let mut push = |tag: &str, value: &str| {
            text.push_str(&format!("|{tag}:{}:{}", value.len(), value));
        };
        push("proposal_id", &self.proposal_id);
        push("execution_id", &self.execution_id);
        push("checkpoint_hash", &self.checkpoint_hash);
        push("group_id", &self.group_id);
        let mut tasks = self.unfinished_task_ids.clone();
        tasks.sort();
        for task in tasks {
            push("unfinished_task", &task);
        }
        for (task, robots) in &self.task_robots {
            let mut robots = robots.clone();
            robots.sort();
            push("task_robots", &format!("{task}={}", robots.join(",")));
        }
        let mut constraints = self.proposal_constraints.clone();
        constraints.sort();
        for constraint in constraints {
            push("constraint", &constraint);
        }
        push("choice_set_certificate_hash", &self.choice_set_certificate_hash);
        push("selected_candidate_hash", &self.selected_candidate_hash);
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

static CONSUMED_AUTHORIZATIONS: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

/// Checkpoint-bound one-shot authorization for research simulation control.
#[derive(Clone, Debug, PartialEq)]
pub struct ReplanAuthorization {
    pub authorization_id: String,
    pub issuer_id: String,
    pub execution_id: String,
    pub checkpoint_hash: String,
    pub proposal_hash: String,
    pub choice_set_certificate_hash: String,
    pub selected_candidate_hash: String,
    pub scope: String,
    pub issued_at_s: f64,
    pub expiration_s: f64,
    pub max_attempts: u32,
    pub signature: String,
}

impl ReplanAuthorization {
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_arguments)]
    fn signing_text(
        id: &str,
        issuer: &str,
        exec: &str,
        cp: &str,
        prop: &str,
        cert: &str,
        candidate: &str,
        scope: &str,
        issued: f64,
        expires: f64,
        attempts: u32,
    ) -> String {
        format!("poie.research-authorization/v2|{id}|{issuer}|{exec}|{cp}|{prop}|{cert}|{candidate}|{scope}|{issued:.6}|{expires:.6}|{attempts}")
    }
    fn signature_for(text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"poie-n21rh-research-issuer-key\x1f");
        hasher.update(text.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    pub fn issue(
        authorization_id: &str,
        checkpoint: &RepairCheckpoint,
        proposal: &ReplanProposal,
        scope: &str,
    ) -> Self {
        let text = Self::signing_text(
            authorization_id,
            "research-issuer",
            &checkpoint.execution_id,
            &checkpoint.state_hash,
            &proposal.compute_hash(),
            &proposal.choice_set_certificate_hash,
            &proposal.selected_candidate_hash,
            scope,
            0.0,
            1.0e12,
            1,
        );
        Self {
            authorization_id: authorization_id.into(),
            issuer_id: "research-issuer".into(),
            execution_id: checkpoint.execution_id.clone(),
            checkpoint_hash: checkpoint.state_hash.clone(),
            proposal_hash: proposal.compute_hash(),
            choice_set_certificate_hash: proposal.choice_set_certificate_hash.clone(),
            selected_candidate_hash: proposal.selected_candidate_hash.clone(),
            scope: scope.into(),
            issued_at_s: 0.0,
            expiration_s: 1.0e12,
            max_attempts: 1,
            signature: Self::signature_for(&text),
        }
    }
    pub fn verify(&self, checkpoint: &RepairCheckpoint, proposal: &ReplanProposal, now_s: f64) -> Result<(), String> {
        if self.issuer_id != "research-issuer" {
            return Err("forged issuer".into());
        }
        if self.max_attempts != 1 {
            return Err("max_attempts must be 1".into());
        }
        if self.expiration_s < now_s {
            return Err("authorization expired".into());
        }
        let text = Self::signing_text(
            &self.authorization_id,
            &self.issuer_id,
            &self.execution_id,
            &self.checkpoint_hash,
            &self.proposal_hash,
            &self.choice_set_certificate_hash,
            &self.selected_candidate_hash,
            &self.scope,
            self.issued_at_s,
            self.expiration_s,
            self.max_attempts,
        );
        if Self::signature_for(&text) != self.signature {
            return Err("invalid authorization signature".into());
        }
        if self.execution_id != checkpoint.execution_id || self.checkpoint_hash != checkpoint.compute_state_hash() {
            return Err("authorization checkpoint mismatch".into());
        }
        if self.proposal_hash != proposal.compute_hash() {
            return Err("authorization proposal mismatch".into());
        }
        if self.scope != "ONE_SHOT_REPLAN" {
            return Err("invalid authorization scope".into());
        }
        Ok(())
    }
}

/// Durable, file-backed one-shot authorization claim ledger.
#[derive(Clone, Debug)]
pub struct FileAuthorizationLedger {
    pub directory: std::path::PathBuf,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LedgerClaim {
    Claimed,
    AlreadyCommitted,
    InProgressOrFailedClosed,
}
impl FileAuthorizationLedger {
    pub fn new(directory: impl Into<std::path::PathBuf>) -> Self {
        Self { directory: directory.into() }
    }
    pub fn claim_id(&self, authorization_id: &str) -> Result<LedgerClaim, String> {
        std::fs::create_dir_all(&self.directory).map_err(|e| e.to_string())?;
        let path = self.directory.join(format!("{authorization_id}.claim"));
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                use std::io::Write;
                file.write_all(b"CLAIMED\n").map_err(|e| e.to_string())?;
                file.sync_all().map_err(|e| e.to_string())?;
                Ok(LedgerClaim::Claimed)
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let text = std::fs::read_to_string(&path).unwrap_or_default();
                if text.starts_with("COMMITTED") {
                    Ok(LedgerClaim::AlreadyCommitted)
                } else {
                    Ok(LedgerClaim::InProgressOrFailedClosed)
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }
    pub fn commit_id(&self, authorization_id: &str, result_identity: &str) -> Result<(), String> {
        let path = self.directory.join(format!("{authorization_id}.claim"));
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, format!("COMMITTED|{result_identity}\n")).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &path).map_err(|e| e.to_string())
    }
}
/// One explicit task in a canonical repair group.
#[derive(Clone, Debug, PartialEq)]
pub struct ExplicitRepairTask {
    pub task_id: String,
    pub process: String,
    pub duration_s: f64,
    /// Optional task-specific robot alternative group. When non-empty the
    /// scheduler chooses exactly one of these resources for this task.
    pub robot_alternatives: Vec<String>,
}

/// Versioned explicit repair-group input. Tasks are scheduled by the canonical
/// scheduler and all complete before the single restoration target is applied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepairLifecycleMode {
    GroupHold,
    TaskRelease,
}

pub struct ExplicitRepairGroup {
    pub group_id: String,
    pub restoration_target: String,
    pub tasks: Vec<ExplicitRepairTask>,
    pub lifecycle: RepairLifecycleMode,
}

#[derive(Clone, Debug)]
enum TimelineItem {
    Failure(FailureEvent),
    Supply(Resupply),
}

struct RunningRepair {
    plans: Vec<Plan>,
    planning_base: BTreeMap<String, f64>,
    execution_dependencies: BTreeSet<String>,
    restoration_targets: BTreeSet<String>,
    resources: Vec<String>,
    witness: Vec<WitnessEntry>,
    task_resources: Vec<TaskResourceObservation>,
    end_s: f64,
    label: String,
    task_release: bool,
    completed_tasks: BTreeSet<usize>,
    task_dependencies: Vec<BTreeSet<String>>,
    released_resources: BTreeSet<String>,
}

enum StartError {
    Blocked(String),
    Incomplete,
}

fn collect_asset_dependencies(model: &Model, asset: &str, seen: &mut BTreeSet<String>) {
    if !seen.insert(asset.to_string()) {
        return;
    }
    let Some(item) = model.assets.get(asset) else { return };
    for required in &item.requires {
        collect_asset_dependencies(model, required, seen);
    }
    if let Some(alternative) = item
        .requires_any
        .iter()
        .find(|candidate| model.assets.get(*candidate).is_some_and(|asset| asset.status == AssetStatus::Available))
    {
        collect_asset_dependencies(model, alternative, seen);
    }
}

fn derive_witness(model: &Model, plans: &[Plan], selected: &[String]) -> Vec<WitnessEntry> {
    let mut witness = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for plan in plans {
        for step in &plan.steps {
            let Some(process) = model.processes.iter().find(|process| process.id == step.process) else { continue };
            for capability in &process.requires_capabilities {
                if witness.iter().any(|entry: &WitnessEntry| entry.capability == *capability) {
                    continue;
                }
                let mut ordered: Vec<&String> = selected
                    .iter()
                    .filter(|candidate| {
                        model.assets.get(*candidate).is_some_and(|asset| asset.provides.contains(capability))
                    })
                    .collect();
                if let Some(providers) = model.providers.get(capability) {
                    for provider in providers {
                        if !ordered.contains(&provider) {
                            ordered.push(provider);
                        }
                    }
                }
                let Some(provider) = ordered.into_iter().find(|provider| {
                    model.assets.get(*provider).is_some_and(|asset| asset.status == AssetStatus::Available)
                }) else {
                    continue;
                };
                let mut dependencies = BTreeSet::new();
                collect_asset_dependencies(model, provider, &mut dependencies);
                dependencies.remove(provider);
                for dependency in &dependencies {
                    seen.insert(dependency.clone());
                }
                seen.insert(provider.clone());
                witness.push(WitnessEntry {
                    capability: capability.clone(),
                    provider: provider.clone(),
                    dependencies: dependencies.into_iter().collect(),
                });
            }
        }
    }
    witness
}

fn start_repair(
    current: &Model,
    pending: &[FailureEvent],
    inventory: &BTreeMap<String, f64>,
    now_s: f64,
    remaining_s: f64,
    resources: &[String],
    alternative_pool: &[String],
) -> Result<RunningRepair, StartError> {
    let planning_base = inventory.clone();
    let planning_model = model_with_inventory(current, &planning_base);
    let mut plans = Vec::new();
    let mut execution_dependencies = BTreeSet::new();
    let mut restoration_targets = BTreeSet::new();
    for failure in pending {
        match analyze(&planning_model, &failure.asset, remaining_s) {
            ClosureOutcome::Restorable(plan) => {
                for step in &plan.steps {
                    if let Some(process) = current.processes.iter().find(|process| process.id == step.process) {
                        for asset in &process.occupies {
                            if !alternative_pool.contains(asset) {
                                execution_dependencies.insert(asset.clone());
                            }
                        }
                    }
                    if let crate::closure::StepAction::Restore(asset) = &step.action {
                        restoration_targets.insert(asset.clone());
                    }
                }
                plans.push(plan);
            }
            ClosureOutcome::Blocked(blocked) => {
                return Err(StartError::Blocked(format!("{:?}", blocked.reasons)));
            }
        }
    }
    let mut pool = resources.to_vec();
    for asset in &execution_dependencies {
        if !pool.contains(asset) && !alternative_pool.contains(asset) {
            pool.push(asset.clone());
        }
    }
    let schedule = match schedule_plans_with_alternatives(
        current,
        &plans,
        &pool,
        alternative_pool,
        std::slice::from_ref(&alternative_pool.to_vec()),
    ) {
        Ok(schedule) => schedule,
        Err(error) => return Err(StartError::Blocked(format!("schedule failed: {error:?}"))),
    };
    if schedule.makespan_s > remaining_s + 1e-9 {
        return Err(StartError::Incomplete);
    }
    for step in &schedule.steps {
        execution_dependencies.extend(step.resources.iter().cloned());
    }
    let selected = schedule.steps.iter().flat_map(|step| step.resources.clone()).collect::<Vec<_>>();
    let witness = derive_witness(current, &plans, &selected);
    for entry in &witness {
        execution_dependencies.insert(entry.provider.clone());
        execution_dependencies.extend(entry.dependencies.iter().cloned());
    }
    let mut task_resources: Vec<TaskResourceObservation> = Vec::new();
    for step in &schedule.steps {
        match task_resources.iter_mut().find(|entry| entry.plan_index == step.plan_index) {
            Some(entry) => {
                for resource in &step.resources {
                    if !entry.resources.contains(resource) {
                        entry.resources.push(resource.clone());
                    }
                }
            }
            None => task_resources.push(TaskResourceObservation {
                plan_index: step.plan_index,
                task_id: None,
                resources: step.resources.clone(),
                start_s: step.start_s,
                end_s: step.end_s,
            }),
        }
    }
    let label = pending.iter().map(|failure| failure.asset.as_str()).collect::<Vec<_>>().join(",");
    Ok(RunningRepair {
        plans,
        planning_base,
        execution_dependencies,
        restoration_targets,
        resources: schedule.steps.iter().flat_map(|step| step.resources.clone()).collect(),
        witness,
        task_resources,
        end_s: now_s + schedule.makespan_s,
        label,
        task_release: false,
        completed_tasks: BTreeSet::new(),
        task_dependencies: Vec::new(),
        released_resources: BTreeSet::new(),
    })
}

fn start_explicit_repair(
    current: &Model,
    group: &ExplicitRepairGroup,
    inventory: &BTreeMap<String, f64>,
    now_s: f64,
    remaining_s: f64,
    resources: &[String],
    alternative_pool: &[String],
) -> Result<RunningRepair, StartError> {
    if group.tasks.is_empty() {
        return Err(StartError::Blocked("explicit repair group has no tasks".into()));
    }
    let planning_base = inventory.clone();
    let mut plans = Vec::new();
    let mut execution_dependencies = BTreeSet::new();
    let mut restoration_targets = BTreeSet::new();
    let mut schedulable = Vec::new();
    let mut has_task_alternatives = false;
    for task in &group.tasks {
        let process = current
            .processes
            .iter()
            .find(|process| process.id == task.process)
            .ok_or_else(|| StartError::Blocked(format!("unknown explicit task process {}", task.process)))?;
        for asset in &process.occupies {
            if !alternative_pool.contains(asset) {
                execution_dependencies.insert(asset.clone());
            }
        }
        restoration_targets.insert(group.restoration_target.clone());
        plans.push(Plan {
            steps: vec![Step {
                process: process.id.clone(),
                action: StepAction::Restore(group.restoration_target.clone()),
                duration_s: task.duration_s,
            }],
            total_duration_s: task.duration_s,
            remaining_inventory: planning_base.clone(),
        });
        let required = process
            .occupies
            .iter()
            .filter(|asset| !task.robot_alternatives.contains(asset))
            .cloned()
            .collect::<Vec<_>>();
        let alternatives =
            if task.robot_alternatives.is_empty() { Vec::new() } else { vec![task.robot_alternatives.clone()] };
        has_task_alternatives |= !alternatives.is_empty();
        schedulable.push(SchedulableTask {
            plan_index: plans.len() - 1,
            step_index: 0,
            process: process.id.clone(),
            duration_s: task.duration_s,
            requirement: ResourceRequirement { required, alternatives },
        });
    }
    let mut pool = resources.to_vec();
    for asset in &execution_dependencies {
        if !pool.contains(asset) && !alternative_pool.contains(asset) {
            pool.push(asset.clone());
        }
    }
    let schedule = if has_task_alternatives {
        match schedule_tasks(&schedulable, &pool) {
            Ok(schedule) => schedule,
            Err(error) => return Err(StartError::Blocked(format!("schedule failed: {error:?}"))),
        }
    } else if alternative_pool.is_empty() {
        match schedule_plans(current, &plans, &pool) {
            Ok(schedule) => schedule,
            Err(error) => return Err(StartError::Blocked(format!("schedule failed: {error:?}"))),
        }
    } else {
        match schedule_plans_with_alternatives(
            current,
            &plans,
            &pool,
            alternative_pool,
            std::slice::from_ref(&alternative_pool.to_vec()),
        ) {
            Ok(schedule) => schedule,
            Err(error) => return Err(StartError::Blocked(format!("schedule failed: {error:?}"))),
        }
    };
    if schedule.makespan_s > remaining_s + 1e-9 {
        return Err(StartError::Incomplete);
    }
    for step in &schedule.steps {
        execution_dependencies.extend(step.resources.iter().cloned());
    }
    let selected = schedule.steps.iter().flat_map(|step| step.resources.clone()).collect::<Vec<_>>();
    let witness = derive_witness(current, &plans, &selected);
    for entry in &witness {
        execution_dependencies.insert(entry.provider.clone());
        execution_dependencies.extend(entry.dependencies.iter().cloned());
    }
    let mut task_resources: Vec<TaskResourceObservation> = Vec::new();
    for step in &schedule.steps {
        match task_resources.iter_mut().find(|entry| entry.plan_index == step.plan_index) {
            Some(entry) => {
                for resource in &step.resources {
                    if !entry.resources.contains(resource) {
                        entry.resources.push(resource.clone());
                    }
                }
            }
            None => task_resources.push(TaskResourceObservation {
                plan_index: step.plan_index,
                task_id: group.tasks.get(step.plan_index).map(|task| task.task_id.clone()),
                resources: step.resources.clone(),
                start_s: step.start_s,
                end_s: step.end_s,
            }),
        }
    }
    // Every actually selected resource and its upstream closure belongs to the
    // initial group closure, including tasks whose capability witness was shared.
    for entry in &task_resources {
        for resource in &entry.resources {
            if resource.starts_with("robot") {
                execution_dependencies.insert(resource.clone());
                let mut deps = BTreeSet::new();
                collect_asset_dependencies(current, resource, &mut deps);
                execution_dependencies.extend(deps);
            }
        }
    }
    let mut task_dependencies: Vec<BTreeSet<String>> = vec![BTreeSet::new(); group.tasks.len()];
    for entry in &task_resources {
        for resource in &entry.resources {
            if resource.starts_with("robot") {
                let mut deps = BTreeSet::new();
                collect_asset_dependencies(current, resource, &mut deps);
                task_dependencies[entry.plan_index].extend(deps);
            }
        }
    }
    Ok(RunningRepair {
        plans,
        planning_base,
        execution_dependencies,
        restoration_targets,
        resources: schedule.steps.iter().flat_map(|step| step.resources.clone()).collect(),
        witness,
        task_resources,
        end_s: now_s + schedule.makespan_s,
        label: group.group_id.clone(),
        task_release: group.lifecycle == RepairLifecycleMode::TaskRelease,
        completed_tasks: BTreeSet::new(),
        task_dependencies,
        released_resources: BTreeSet::new(),
    })
}

fn plan_summary(plans: &[Plan]) -> Vec<String> {
    plans.iter().map(|plan| plan.steps.iter().map(|step| step.process.clone()).collect::<Vec<_>>().join("+")).collect()
}

/// Runs the canonical kernel.
pub fn run_engine(model: &Model, options: &EngineOptions<'_>) -> Result<EngineOutcome, EngineError> {
    run_engine_inner(model, options, None, None)
}

/// Runs the canonical kernel with one explicit multi-task repair group while
/// reusing the same event loop, ledger, failure handling and termination rules.
pub fn run_engine_with_repair_group(
    model: &Model,
    options: &EngineOptions<'_>,
    group: &ExplicitRepairGroup,
) -> Result<EngineOutcome, EngineError> {
    run_engine_inner(model, options, Some(group), None)
}

pub fn run_engine_with_repair_group_event_observed(
    model: &Model,
    options: &EngineOptions<'_>,
    group: &ExplicitRepairGroup,
) -> Result<EventObservedOutcome, EngineError> {
    let guard = EventGuard::install()?;
    let outcome = run_engine_inner(model, options, Some(group), None);
    let events = EVENT_COLLECTOR.with(|cell| cell.borrow_mut().take()).unwrap_or_default();
    drop(guard);
    outcome.map(|outcome| EventObservedOutcome { outcome, events })
}

/// Opt-in v11: run an explicit repair group and capture a native checkpoint at
/// the first repair interruption. Default `run_engine*` behavior is unchanged.
pub fn run_engine_with_repair_group_to_checkpoint(
    model: &Model,
    options: &EngineOptions<'_>,
    group: &ExplicitRepairGroup,
    execution_id: &str,
) -> Result<(EngineOutcome, Option<RepairCheckpoint>), EngineError> {
    let mut checkpoint = None;
    let outcome = run_engine_inner(model, options, Some(group), Some((execution_id, &mut checkpoint)))?;
    Ok((outcome, checkpoint))
}

/// Opt-in v11: resume the same execution from a checkpoint under one valid,
/// checkpoint-bound one-shot authorization and a bounded replan proposal.
pub fn resume_engine_with_authorization(
    model: &Model,
    options: &EngineOptions<'_>,
    group: &ExplicitRepairGroup,
    checkpoint: &RepairCheckpoint,
    authorization: &ReplanAuthorization,
    proposal: &ReplanProposal,
) -> Result<EngineOutcome, EngineError> {
    if authorization.verify(checkpoint, proposal, checkpoint.at_s).is_err() {
        return Ok(EngineOutcome {
            termination: EngineTermination::Blocked("replan_rejected: invalid authorization".into()),
            survived_s: checkpoint.at_s,
            trace: checkpoint.trace_history.clone(),
            final_inventory: checkpoint.inventory.clone(),
        });
    }
    for failure in &checkpoint.applied_failures {
        if checkpoint.asset_statuses.get(&failure.asset) != Some(&AssetStatus::Failed) {
            return Ok(EngineOutcome {
                termination: EngineTermination::Blocked(format!(
                    "replan_rejected: checkpoint lost failure history for {}",
                    failure.asset
                )),
                survived_s: checkpoint.at_s,
                trace: checkpoint.trace_history.clone(),
                final_inventory: checkpoint.inventory.clone(),
            });
        }
    }
    {
        let mut consumed =
            CONSUMED_AUTHORIZATIONS.lock().map_err(|_| EngineError("authorization registry poisoned".into()))?;
        if !consumed.insert(authorization.authorization_id.clone()) {
            return Ok(EngineOutcome {
                termination: EngineTermination::Blocked("replan_rejected: authorization already consumed".into()),
                survived_s: checkpoint.at_s,
                trace: checkpoint.trace_history.clone(),
                final_inventory: checkpoint.inventory.clone(),
            });
        }
    }
    if group.group_id != checkpoint.group_id {
        return Ok(EngineOutcome {
            termination: EngineTermination::Blocked("replan_rejected: group mismatch".into()),
            survived_s: checkpoint.at_s,
            trace: checkpoint.trace_history.clone(),
            final_inventory: checkpoint.inventory.clone(),
        });
    }
    let mut current = model.clone();
    for (asset, status) in &checkpoint.asset_statuses {
        if let Some(item) = current.assets.get_mut(asset) {
            item.status = *status;
        }
    }
    let mut inventory = checkpoint.inventory.clone();
    if !proposal.execution_id.is_empty() && proposal.execution_id != checkpoint.execution_id {
        return Ok(EngineOutcome {
            termination: EngineTermination::Blocked("replan_rejected: proposal execution mismatch".into()),
            survived_s: checkpoint.at_s,
            trace: checkpoint.trace_history.clone(),
            final_inventory: inventory,
        });
    }
    if !proposal.checkpoint_hash.is_empty() && proposal.checkpoint_hash != checkpoint.compute_state_hash() {
        return Ok(EngineOutcome {
            termination: EngineTermination::Blocked("replan_rejected: proposal checkpoint mismatch".into()),
            survived_s: checkpoint.at_s,
            trace: checkpoint.trace_history.clone(),
            final_inventory: inventory,
        });
    }
    if !proposal.group_id.is_empty() && proposal.group_id != checkpoint.group_id {
        return Ok(EngineOutcome {
            termination: EngineTermination::Blocked("replan_rejected: proposal group mismatch".into()),
            survived_s: checkpoint.at_s,
            trace: checkpoint.trace_history.clone(),
            final_inventory: inventory,
        });
    }
    if !proposal.unfinished_task_ids.is_empty() && {
        let mut a = proposal.unfinished_task_ids.clone();
        a.sort();
        let mut b = checkpoint.unfinished_tasks.clone();
        b.sort();
        a != b
    } {
        return Ok(EngineOutcome {
            termination: EngineTermination::Blocked("replan_rejected: proposal unfinished tasks mismatch".into()),
            survived_s: checkpoint.at_s,
            trace: checkpoint.trace_history.clone(),
            final_inventory: inventory,
        });
    }
    if proposal.selected_candidate_hash.is_empty() || proposal.choice_set_certificate_hash.is_empty() {
        return Ok(EngineOutcome {
            termination: EngineTermination::Blocked(
                "replan_rejected: proposal not bound to choice-set candidate".into(),
            ),
            survived_s: checkpoint.at_s,
            trace: checkpoint.trace_history.clone(),
            final_inventory: inventory,
        });
    }
    for (task_id, robots) in &proposal.task_robots {
        if !checkpoint.unfinished_tasks.contains(task_id) {
            return Ok(EngineOutcome {
                termination: EngineTermination::Blocked(format!(
                    "replan_rejected: completed task {task_id} cannot be replayed"
                )),
                survived_s: checkpoint.at_s,
                trace: checkpoint.trace_history.clone(),
                final_inventory: inventory,
            });
        }
        for robot in robots {
            if current.assets.get(robot).is_none_or(|asset| asset.status != AssetStatus::Available) {
                return Ok(EngineOutcome {
                    termination: EngineTermination::Blocked(format!("replan_rejected: robot {robot} is not available")),
                    survived_s: checkpoint.at_s,
                    trace: checkpoint.trace_history.clone(),
                    final_inventory: inventory,
                });
            }
        }
    }
    let remaining_tasks: Vec<ExplicitRepairTask> = group
        .tasks
        .iter()
        .filter(|task| checkpoint.unfinished_tasks.contains(&task.task_id))
        .map(|task| {
            let mut task = task.clone();
            if let Some(robots) = proposal.task_robots.get(&task.task_id) {
                task.robot_alternatives = robots.clone();
            }
            task
        })
        .collect();
    if remaining_tasks.is_empty() {
        return Ok(EngineOutcome {
            termination: EngineTermination::Blocked("replan_rejected: no unfinished tasks".into()),
            survived_s: checkpoint.at_s,
            trace: checkpoint.trace_history.clone(),
            final_inventory: inventory,
        });
    }
    let subgroup = ExplicitRepairGroup {
        group_id: group.group_id.clone(),
        restoration_target: group.restoration_target.clone(),
        tasks: remaining_tasks,
        lifecycle: group.lifecycle,
    };
    let running = match start_explicit_repair(
        &current,
        &subgroup,
        &inventory,
        checkpoint.at_s,
        checkpoint.remaining_horizon_s,
        &options.resources,
        &options.alternative_pool,
    ) {
        Ok(repair) => repair,
        Err(StartError::Blocked(reason)) => {
            return Ok(EngineOutcome {
                termination: EngineTermination::Blocked(format!("replan_rejected: {reason}")),
                survived_s: checkpoint.at_s,
                trace: checkpoint.trace_history.clone(),
                final_inventory: inventory,
            })
        }
        Err(StartError::Incomplete) => {
            return Ok(EngineOutcome {
                termination: EngineTermination::IncompleteAtHorizon("replan".into()),
                survived_s: checkpoint.at_s + checkpoint.remaining_horizon_s,
                trace: checkpoint.trace_history.clone(),
                final_inventory: inventory,
            })
        }
    };
    let end_s = running.end_s;
    let horizon_s = checkpoint.at_s + checkpoint.remaining_horizon_s;
    if end_s > horizon_s + 1e-9 {
        return Ok(EngineOutcome {
            termination: EngineTermination::IncompleteAtHorizon("replan at horizon".into()),
            survived_s: horizon_s,
            trace: checkpoint.trace_history.clone(),
            final_inventory: inventory,
        });
    }
    let mut trace = checkpoint.trace_history.clone();
    let mut event_sequence = checkpoint.event_sequence;
    let mut event_batch = checkpoint.event_batch_sequence;
    let snapshot = asset_status_snapshot(&current.assets);
    event_sequence += 1;
    event_batch += 1;
    record_event(
        event_sequence,
        event_batch,
        checkpoint.at_s,
        EventKind::ReplanAuthorized,
        EventOrigin::Exogenous,
        Some(authorization.authorization_id.clone()),
        EventStatus::Committed,
        &snapshot,
        &snapshot,
        &inventory,
        &inventory,
        Some(&running),
    );
    event_sequence += 1;
    event_batch += 1;
    record_event(
        event_sequence,
        event_batch,
        checkpoint.at_s,
        EventKind::RepairResumed,
        EventOrigin::Endogenous,
        Some(group.group_id.clone()),
        EventStatus::Committed,
        &snapshot,
        &snapshot,
        &inventory,
        &inventory,
        Some(&running),
    );
    for task in &running.task_resources {
        if let Some(task_id) = &task.task_id {
            event_sequence += 1;
            event_batch += 1;
            record_event(
                event_sequence,
                event_batch,
                checkpoint.at_s,
                EventKind::TaskRestarted,
                EventOrigin::Endogenous,
                Some(task_id.clone()),
                EventStatus::Committed,
                &snapshot,
                &snapshot,
                &inventory,
                &inventory,
                Some(&running),
            );
        }
    }
    let mut failures = checkpoint.remaining_failures.clone();
    failures.sort_by(|a, b| a.at_s.total_cmp(&b.at_s));
    for failure in failures {
        if failure.at_s < end_s - 1e-9 {
            let before = asset_status_snapshot(&current.assets);
            if let Some(asset) = current.assets.get_mut(&failure.asset) {
                asset.status = AssetStatus::Failed;
            }
            event_sequence += 1;
            event_batch += 1;
            let after = asset_status_snapshot(&current.assets);
            record_event(
                event_sequence,
                event_batch,
                failure.at_s,
                EventKind::AssetFailure,
                EventOrigin::Exogenous,
                Some(failure.asset.clone()),
                EventStatus::Committed,
                &before,
                &after,
                &inventory,
                &inventory,
                Some(&running),
            );
            if running.execution_dependencies.contains(&failure.asset)
                || running.restoration_targets.contains(&failure.asset)
            {
                event_sequence += 1;
                event_batch += 1;
                record_event(
                    event_sequence,
                    event_batch,
                    failure.at_s,
                    EventKind::RepairInterrupted,
                    EventOrigin::Endogenous,
                    Some("repair".into()),
                    EventStatus::Committed,
                    &after,
                    &after,
                    &inventory,
                    &inventory,
                    Some(&running),
                );
                return Ok(EngineOutcome {
                    termination: EngineTermination::Blocked(format!(
                        "continuation interrupted at {:.0}s: {} failed",
                        failure.at_s, failure.asset
                    )),
                    survived_s: failure.at_s,
                    trace,
                    final_inventory: inventory,
                });
            }
        }
    }
    let mut committed = current.clone();
    let mut committed_inventory = inventory.clone();
    if let Err(reason) = apply_plans(
        &mut committed,
        &mut committed_inventory,
        &checkpoint.completed_plans,
        &checkpoint.completed_planning_base,
    ) {
        return Ok(EngineOutcome {
            termination: EngineTermination::Blocked(format!("replan_commit_failed: {reason}")),
            survived_s: checkpoint.at_s,
            trace,
            final_inventory: inventory,
        });
    }
    if let Err(reason) = apply_plans(&mut committed, &mut committed_inventory, &running.plans, &running.planning_base) {
        return Ok(EngineOutcome {
            termination: EngineTermination::Blocked(format!("replan_commit_failed: {reason}")),
            survived_s: checkpoint.at_s,
            trace,
            final_inventory: inventory,
        });
    }
    current = committed;
    inventory = committed_inventory;
    let after = asset_status_snapshot(&current.assets);
    for task in &running.task_resources {
        if let Some(task_id) = &task.task_id {
            event_sequence += 1;
            event_batch += 1;
            record_event(
                event_sequence,
                event_batch,
                end_s,
                EventKind::TaskCompleted,
                EventOrigin::Endogenous,
                Some(task_id.clone()),
                EventStatus::Committed,
                &after,
                &after,
                &inventory,
                &inventory,
                None,
            );
        }
    }
    event_sequence += 1;
    event_batch += 1;
    record_event(
        event_sequence,
        event_batch,
        end_s,
        EventKind::RepairCompleted,
        EventOrigin::Endogenous,
        Some(group.group_id.clone()),
        EventStatus::Committed,
        &after,
        &after,
        &inventory,
        &inventory,
        None,
    );
    event_sequence += 1;
    event_batch += 1;
    record_event(
        event_sequence,
        event_batch,
        end_s,
        EventKind::RestorationCommitted,
        EventOrigin::Endogenous,
        Some(group.restoration_target.clone()),
        EventStatus::Committed,
        &after,
        &after,
        &inventory,
        &inventory,
        None,
    );
    trace.push(EngineTrace {
        at_s: end_s,
        kind: format!("replan_completed:{}", group.group_id),
        inventory_before: checkpoint.inventory.clone(),
        inventory_after: inventory.clone(),
        plans: plan_summary(&running.plans),
        resources: running.resources.clone(),
        witness: running.witness.clone(),
        energy_generated_j: 0.0,
        energy_resupply_j: 0.0,
        energy_load_j: 0.0,
        energy_repair_j: 0.0,
        energy_curtailed_j: 0.0,
    });
    Ok(EngineOutcome {
        termination: EngineTermination::CompletedHorizon,
        survived_s: end_s,
        trace,
        final_inventory: inventory,
    })
}

/// Durable wrapper: atomically claim the authorization in a file ledger
/// before resuming, then commit the result identity. A crash after claim but
/// before commit leaves the authorization fail-closed.
pub fn resume_engine_with_durable_authorization(
    model: &Model,
    options: &EngineOptions<'_>,
    group: &ExplicitRepairGroup,
    checkpoint: &RepairCheckpoint,
    authorization: &ReplanAuthorization,
    proposal: &ReplanProposal,
    ledger: &FileAuthorizationLedger,
) -> Result<EngineOutcome, EngineError> {
    authorization.verify(checkpoint, proposal, checkpoint.at_s).map_err(EngineError)?;
    match ledger.claim_id(&authorization.authorization_id).map_err(EngineError)? {
        LedgerClaim::Claimed => {
            let outcome = resume_engine_with_authorization(model, options, group, checkpoint, authorization, proposal)?;
            if matches!(outcome.termination, EngineTermination::CompletedHorizon) {
                ledger
                    .commit_id(&authorization.authorization_id, &format!("{:.6}", outcome.survived_s))
                    .map_err(EngineError)?;
            }
            Ok(outcome)
        }
        LedgerClaim::AlreadyCommitted => Ok(EngineOutcome {
            termination: EngineTermination::Blocked("replan_rejected: authorization already committed".into()),
            survived_s: checkpoint.at_s,
            trace: checkpoint.trace_history.clone(),
            final_inventory: checkpoint.inventory.clone(),
        }),
        LedgerClaim::InProgressOrFailedClosed => Ok(EngineOutcome {
            termination: EngineTermination::Blocked("replan_rejected: authorization claimed or failed closed".into()),
            survived_s: checkpoint.at_s,
            trace: checkpoint.trace_history.clone(),
            final_inventory: checkpoint.inventory.clone(),
        }),
    }
}

fn run_engine_inner(
    model: &Model,
    options: &EngineOptions<'_>,
    explicit_group: Option<&ExplicitRepairGroup>,
    mut capture: Option<(&str, &mut Option<RepairCheckpoint>)>,
) -> Result<EngineOutcome, EngineError> {
    let mut services: Vec<&Service> = Vec::new();
    for service_id in &options.services {
        let Some(service) = model.services.get(*service_id) else {
            return Err(EngineError(format!("unknown service {service_id}")));
        };
        services.push(service);
    }
    let mut current = model.clone();
    let mut inventory = initial_inventory(model);
    if let Some(battery) = options.battery {
        let capacity_j = battery.capacity_kwh * 3_600_000.0;
        if energy(&inventory) > capacity_j + 1e-9 {
            return Err(EngineError(format!(
                "initial electric-energy {:.0} J exceeds battery capacity {capacity_j:.0} J",
                energy(&inventory)
            )));
        }
    }
    let mut clock = EventClock::new(options.horizon_s);
    let mut trace = Vec::new();
    let mut interval_flow = crate::survival::EnergyFlow::default();
    let mut event_sequence = 0usize;
    let mut event_batch = 0usize;

    // Initial repair is its own event.
    let planning_model = model_with_inventory(&current, &inventory);
    let mut initial_assets: Vec<String> = Vec::new();
    for service in &services {
        for asset in &service.requires {
            if current.assets.get(asset).is_some_and(|item| item.status != AssetStatus::Available) {
                initial_assets.push(asset.clone());
            }
        }
        if !service.requires_any.is_empty()
            && !service
                .requires_any
                .iter()
                .any(|asset| current.assets.get(asset).is_some_and(|item| item.status == AssetStatus::Available))
        {
            for asset in &service.requires_any {
                if analyze(&planning_model, asset, options.horizon_s).is_restorable() {
                    initial_assets.push(asset.clone());
                    break;
                }
            }
        }
    }
    initial_assets.sort();
    initial_assets.dedup();
    let mut pending: Vec<FailureEvent> =
        initial_assets.into_iter().map(|asset| FailureEvent { at_s: 0.0, asset }).collect();
    let mut running: Option<RunningRepair> = None;
    let initial_attempt = if let Some(group) = explicit_group {
        start_explicit_repair(
            &current,
            group,
            &inventory,
            clock.now_s(),
            clock.remaining_s(),
            &options.resources,
            &options.alternative_pool,
        )
    } else if !pending.is_empty() {
        start_repair(
            &current,
            &pending,
            &inventory,
            clock.now_s(),
            clock.remaining_s(),
            &options.resources,
            &options.alternative_pool,
        )
    } else {
        Err(StartError::Incomplete)
    };
    if explicit_group.is_some() || !pending.is_empty() {
        match initial_attempt {
            Ok(repair) => {
                running = Some(repair);
                pending.clear();
                let snapshot = asset_status_snapshot(&current.assets);
                event_sequence += 1;
                event_batch += 1;
                record_event(
                    event_sequence,
                    event_batch,
                    clock.now_s(),
                    EventKind::InitialRepairStarted,
                    EventOrigin::Endogenous,
                    Some("initial-repair".into()),
                    EventStatus::Committed,
                    &snapshot,
                    &snapshot,
                    &inventory,
                    &inventory,
                    running.as_ref(),
                );
            }
            Err(StartError::Blocked(reason)) => {
                return Ok(EngineOutcome {
                    termination: EngineTermination::Blocked(reason),
                    survived_s: 0.0,
                    trace,
                    final_inventory: inventory,
                });
            }
            Err(StartError::Incomplete) => {
                return Ok(EngineOutcome {
                    termination: EngineTermination::IncompleteAtHorizon("initial repair".into()),
                    survived_s: options.horizon_s,
                    trace,
                    final_inventory: inventory,
                });
            }
        }
    }

    observe_due(
        clock.now_s(),
        ObservationPhase::After,
        &inventory,
        running.as_ref(),
        &trace,
        &interval_flow,
        &asset_status_snapshot(&current.assets),
    );

    let mut timeline: Vec<(f64, TimelineItem)> = Vec::new();
    for failure in &options.failures {
        timeline.push((failure.at_s, TimelineItem::Failure(failure.clone())));
    }
    for supply in &options.resupplies {
        timeline.push((supply.at_s, TimelineItem::Supply(supply.clone())));
    }
    timeline.sort_by(|left, right| {
        left.0.total_cmp(&right.0).then_with(|| {
            matches!(left.1, TimelineItem::Supply(_)).cmp(&matches!(right.1, TimelineItem::Supply(_))).reverse()
        })
    });

    let mut index = 0usize;
    let mut initial_phase = running.is_some();
    loop {
        if running.is_none() && !pending.is_empty() {
            match start_repair(
                &current,
                &pending,
                &inventory,
                clock.now_s(),
                clock.remaining_s(),
                &options.resources,
                &options.alternative_pool,
            ) {
                Ok(repair) => {
                    running = Some(repair);
                    pending.clear();
                    let snapshot = asset_status_snapshot(&current.assets);
                    event_sequence += 1;
                    event_batch += 1;
                    record_event(
                        event_sequence,
                        event_batch,
                        clock.now_s(),
                        EventKind::RepairStarted,
                        EventOrigin::Endogenous,
                        Some("repair".into()),
                        EventStatus::Committed,
                        &snapshot,
                        &snapshot,
                        &inventory,
                        &inventory,
                        running.as_ref(),
                    );
                }
                Err(StartError::Blocked(reason)) => {
                    return Ok(EngineOutcome {
                        termination: EngineTermination::Blocked(reason),
                        survived_s: clock.now_s(),
                        trace,
                        final_inventory: inventory,
                    });
                }
                Err(StartError::Incomplete) => {
                    return Ok(EngineOutcome {
                        termination: EngineTermination::IncompleteAtHorizon("repair in flight".into()),
                        survived_s: options.horizon_s,
                        trace,
                        final_inventory: inventory,
                    });
                }
            }
        }
        let raw_next_event = timeline.get(index).map(|(at_s, _)| (*at_s).max(clock.now_s()));
        let next_event = if initial_phase {
            raw_next_event.map(|at_s| {
                if at_s <= clock.now_s() + 1e-9 {
                    running.as_ref().map(|repair| repair.end_s).unwrap_or(at_s)
                } else {
                    at_s
                }
            })
        } else {
            raw_next_event
        };
        let next_completion = running.as_ref().map(|repair| {
            if repair.task_release {
                repair
                    .task_resources
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| !repair.completed_tasks.contains(index))
                    .map(|(_, task)| task.end_s)
                    .fold(f64::INFINITY, f64::min)
            } else {
                repair.end_s
            }
        });
        let mut next_s =
            [next_event, next_completion, Some(options.horizon_s)].into_iter().flatten().fold(f64::INFINITY, f64::min);
        let mut sample_only = false;
        if let Some(request_s) = next_request_time() {
            if request_s >= clock.now_s() - 1e-9 && request_s <= next_s + 1e-9 {
                sample_only = request_s < next_s - 1e-9;
                next_s = request_s;
            }
        }
        if next_s > clock.horizon_s() + 1e-9 {
            break;
        }
        let gap = clock.advance_to(next_s);
        match apply_energy_interval_ledger(&current, &mut inventory, gap, options.battery, options.load_kw) {
            Ok(flow) => {
                interval_flow.generated_j += flow.generated_j;
                interval_flow.load_j += flow.load_j;
                interval_flow.curtailed_j += flow.curtailed_j;
            }
            Err(offset) => {
                let assets = asset_status_snapshot(&current.assets);
                event_sequence += 1;
                record_event(
                    event_sequence,
                    event_batch + 1,
                    clock.now_s() - gap + offset,
                    EventKind::EnergyDepleted,
                    EventOrigin::Endogenous,
                    None,
                    EventStatus::Committed,
                    &assets,
                    &assets,
                    &inventory,
                    &inventory,
                    running.as_ref(),
                );
                return Ok(EngineOutcome {
                    termination: EngineTermination::Blocked("energy_depleted".into()),
                    survived_s: clock.now_s() - gap + offset,
                    trace,
                    final_inventory: inventory,
                });
            }
        }
        observe_due(
            clock.now_s(),
            ObservationPhase::Before,
            &inventory,
            running.as_ref(),
            &trace,
            &interval_flow,
            &asset_status_snapshot(&current.assets),
        );
        if sample_only {
            observe_due(
                clock.now_s(),
                ObservationPhase::After,
                &inventory,
                running.as_ref(),
                &trace,
                &interval_flow,
                &asset_status_snapshot(&current.assets),
            );
            continue;
        }
        let mut completed_events: Vec<String> = Vec::new();
        let mut released_events: Vec<String> = Vec::new();
        if let Some(repair) = running.as_mut() {
            if repair.task_release {
                for (index, task) in repair.task_resources.iter().enumerate() {
                    if task.end_s <= clock.now_s() + 1e-9 && !repair.completed_tasks.contains(&index) {
                        repair.completed_tasks.insert(index);
                        completed_events.push(task.task_id.clone().unwrap_or_else(|| format!("task-{index}")));
                    }
                }
                let unfinished: Vec<usize> = repair
                    .task_resources
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| !repair.completed_tasks.contains(index))
                    .map(|(index, _)| index)
                    .collect();
                let mut needed = BTreeSet::new();
                let mut needed_deps = BTreeSet::new();
                for index in &unfinished {
                    needed.extend(repair.task_resources[*index].resources.iter().cloned());
                    if let Some(deps) = repair.task_dependencies.get(*index) {
                        needed_deps.extend(deps.iter().cloned());
                    }
                }
                let all_resources: BTreeSet<String> =
                    repair.task_resources.iter().flat_map(|task| task.resources.iter().cloned()).collect();
                for resource in all_resources {
                    if !needed.contains(&resource) && repair.execution_dependencies.remove(&resource) {
                        repair.released_resources.insert(resource.clone());
                        released_events.push(resource);
                    }
                }
                let removable_deps: Vec<String> = repair
                    .execution_dependencies
                    .iter()
                    .filter(|dep| {
                        !needed_deps.contains(*dep) && repair.task_dependencies.iter().flatten().any(|d| d == *dep)
                    })
                    .cloned()
                    .collect();
                for dep in removable_deps {
                    if repair.execution_dependencies.remove(&dep) {
                        repair.released_resources.insert(dep.clone());
                        released_events.push(dep);
                    }
                }
            }
        }
        for task_id in completed_events {
            let snapshot = asset_status_snapshot(&current.assets);
            event_sequence += 1;
            event_batch += 1;
            record_event(
                event_sequence,
                event_batch,
                clock.now_s(),
                EventKind::TaskCompleted,
                EventOrigin::Endogenous,
                Some(task_id),
                EventStatus::Committed,
                &snapshot,
                &snapshot,
                &inventory,
                &inventory,
                running.as_ref(),
            );
        }
        for resource in released_events {
            let snapshot = asset_status_snapshot(&current.assets);
            event_sequence += 1;
            event_batch += 1;
            record_event(
                event_sequence,
                event_batch,
                clock.now_s(),
                EventKind::ResourceReleased,
                EventOrigin::Endogenous,
                Some(resource),
                EventStatus::Committed,
                &snapshot,
                &snapshot,
                &inventory,
                &inventory,
                running.as_ref(),
            );
        }
        if let Some(repair) = running.as_ref() {
            if repair.end_s <= clock.now_s() + 1e-9 {
                let repair = running.take().expect("running repair");
                initial_phase = false;
                let before = inventory.clone();
                let assets_before = asset_status_snapshot(&current.assets);
                if let Err(reason) = apply_plans(&mut current, &mut inventory, &repair.plans, &repair.planning_base) {
                    return Ok(EngineOutcome {
                        termination: EngineTermination::Blocked(reason),
                        survived_s: clock.now_s(),
                        trace,
                        final_inventory: inventory,
                    });
                }
                let assets_after = asset_status_snapshot(&current.assets);
                event_sequence += 1;
                event_batch += 1;
                record_event(
                    event_sequence,
                    event_batch,
                    clock.now_s(),
                    EventKind::RepairCompleted,
                    EventOrigin::Endogenous,
                    Some(repair.label.clone()),
                    EventStatus::Committed,
                    &assets_before,
                    &assets_after,
                    &before,
                    &inventory,
                    None,
                );
                let repair_j: f64 = repair
                    .plans
                    .iter()
                    .map(|plan| {
                        let before = repair.planning_base.get("electric-energy").copied().unwrap_or(0.0);
                        let after = plan.remaining_inventory.get("electric-energy").copied().unwrap_or(0.0);
                        (before - after).max(0.0)
                    })
                    .sum();
                let flow = std::mem::take(&mut interval_flow);
                trace.push(EngineTrace {
                    at_s: clock.now_s(),
                    kind: format!("repair_completed:{}", repair.label),
                    inventory_before: before.clone(),
                    inventory_after: inventory.clone(),
                    plans: plan_summary(&repair.plans),
                    resources: repair.resources.clone(),
                    witness: repair.witness.clone(),
                    energy_generated_j: flow.generated_j,
                    energy_resupply_j: 0.0,
                    energy_load_j: flow.load_j,
                    energy_repair_j: repair_j,
                    energy_curtailed_j: flow.curtailed_j,
                });
                observe_due(
                    clock.now_s(),
                    ObservationPhase::After,
                    &inventory,
                    running.as_ref(),
                    &trace,
                    &interval_flow,
                    &asset_status_snapshot(&current.assets),
                );
                continue;
            }
        }
        if clock.now_s() >= options.horizon_s - 1e-9 {
            observe_due(
                clock.now_s(),
                ObservationPhase::After,
                &inventory,
                running.as_ref(),
                &trace,
                &interval_flow,
                &asset_status_snapshot(&current.assets),
            );
            if running.is_some() {
                return Ok(EngineOutcome {
                    termination: EngineTermination::IncompleteAtHorizon("repair at horizon".into()),
                    survived_s: options.horizon_s,
                    trace,
                    final_inventory: inventory,
                });
            }
            break;
        }
        // Process all events at this timestamp: resupply first, then failures.
        let batch_start = index;
        while index < timeline.len() && timeline[index].0 <= clock.now_s() + 1e-9 {
            index += 1;
        }
        let mut failed: Vec<FailureEvent> = Vec::new();
        let mut repairable: Vec<FailureEvent> = Vec::new();
        event_batch += 1;
        for (_, item) in &timeline[batch_start..index] {
            if let TimelineItem::Supply(supply) = item {
                let before = inventory.clone();
                *inventory.entry(supply.resource.clone()).or_insert(0.0) += supply.si;
                let mut resupply_j = if supply.resource == "electric-energy" { supply.si } else { 0.0 };
                let mut curtailed_j = 0.0;
                if let Some(battery) = options.battery {
                    let capacity_j = battery.capacity_kwh * 3_600_000.0;
                    if let Some(stored) = inventory.get_mut("electric-energy") {
                        if *stored > capacity_j {
                            curtailed_j = *stored - capacity_j;
                            *stored = capacity_j;
                        }
                    }
                }
                if resupply_j > 0.0 && curtailed_j > 0.0 {
                    resupply_j -= curtailed_j;
                }
                let flow = std::mem::take(&mut interval_flow);
                trace.push(EngineTrace {
                    at_s: clock.now_s(),
                    kind: format!("resupply:{}", supply.resource),
                    inventory_before: before.clone(),
                    inventory_after: inventory.clone(),
                    plans: Vec::new(),
                    resources: Vec::new(),
                    witness: Vec::new(),
                    energy_generated_j: flow.generated_j,
                    energy_resupply_j: resupply_j,
                    energy_load_j: flow.load_j,
                    energy_repair_j: 0.0,
                    energy_curtailed_j: flow.curtailed_j + curtailed_j,
                });
                event_sequence += 1;
                let assets = asset_status_snapshot(&current.assets);
                record_event(
                    event_sequence,
                    event_batch,
                    clock.now_s(),
                    EventKind::Resupply,
                    EventOrigin::Exogenous,
                    Some(supply.resource.clone()),
                    EventStatus::Committed,
                    &assets,
                    &assets,
                    &before,
                    &inventory,
                    running.as_ref(),
                );
            }
        }
        for (_, item) in &timeline[batch_start..index] {
            if let TimelineItem::Failure(failure) = item {
                let assets_before = asset_status_snapshot(&current.assets);
                match current.assets.get_mut(&failure.asset) {
                    Some(asset) => asset.status = AssetStatus::Failed,
                    None => {
                        return Ok(EngineOutcome {
                            termination: EngineTermination::Blocked(format!("unknown asset {}", failure.asset)),
                            survived_s: clock.now_s(),
                            trace,
                            final_inventory: inventory,
                        });
                    }
                }
                let before = inventory.clone();
                let flow = std::mem::take(&mut interval_flow);
                trace.push(EngineTrace {
                    at_s: clock.now_s(),
                    kind: format!("failure:{}", failure.asset),
                    inventory_before: before.clone(),
                    inventory_after: inventory.clone(),
                    plans: Vec::new(),
                    resources: Vec::new(),
                    witness: Vec::new(),
                    energy_generated_j: flow.generated_j,
                    energy_resupply_j: 0.0,
                    energy_load_j: flow.load_j,
                    energy_repair_j: 0.0,
                    energy_curtailed_j: flow.curtailed_j,
                });
                event_sequence += 1;
                let assets_after = asset_status_snapshot(&current.assets);
                record_event(
                    event_sequence,
                    event_batch,
                    clock.now_s(),
                    EventKind::AssetFailure,
                    EventOrigin::Exogenous,
                    Some(failure.asset.clone()),
                    EventStatus::Committed,
                    &assets_before,
                    &assets_after,
                    &before,
                    &inventory,
                    running.as_ref(),
                );
                let relevant = services.iter().any(|service| {
                    if service.requires.contains(&failure.asset) {
                        return true;
                    }
                    if !service.requires_any.contains(&failure.asset) {
                        return false;
                    }
                    !service.requires_any.iter().any(|alternative| {
                        alternative != &failure.asset
                            && current
                                .assets
                                .get(alternative)
                                .is_some_and(|asset| asset.status == AssetStatus::Available)
                    })
                });
                failed.push(failure.clone());
                if relevant {
                    repairable.push(failure.clone());
                }
            }
        }
        if let Some(repair) = running.as_ref() {
            if let Some(hit) = failed.iter().find(|failure| {
                repair.execution_dependencies.contains(&failure.asset)
                    || repair.restoration_targets.contains(&failure.asset)
            }) {
                observe_due(
                    clock.now_s(),
                    ObservationPhase::After,
                    &inventory,
                    running.as_ref(),
                    &trace,
                    &interval_flow,
                    &asset_status_snapshot(&current.assets),
                );
                let snapshot = asset_status_snapshot(&current.assets);
                event_sequence += 1;
                record_event(
                    event_sequence,
                    event_batch,
                    clock.now_s(),
                    EventKind::RepairInterrupted,
                    EventOrigin::Endogenous,
                    Some("repair".into()),
                    EventStatus::Committed,
                    &snapshot,
                    &snapshot,
                    &inventory,
                    &inventory,
                    running.as_ref(),
                );
                let interrupted_trace = trace;
                if let Some((execution_id, slot)) = capture.as_mut() {
                    let repair = running.as_ref().expect("running repair");
                    let completed_tasks: Vec<String> = repair
                        .task_resources
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| repair.completed_tasks.contains(index))
                        .filter_map(|(_, task)| task.task_id.clone())
                        .collect();
                    let unfinished_tasks: Vec<String> = repair
                        .task_resources
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| !repair.completed_tasks.contains(index))
                        .filter_map(|(_, task)| task.task_id.clone())
                        .collect();
                    let mut selected_resources = BTreeMap::new();
                    for task in &repair.task_resources {
                        selected_resources
                            .insert(task.task_id.clone().unwrap_or_else(|| "task".into()), task.resources.clone());
                    }
                    let applied_failures: Vec<FailureEvent> = timeline[..index]
                        .iter()
                        .filter_map(|(_, item)| {
                            if let TimelineItem::Failure(failure) = item {
                                Some(failure.clone())
                            } else {
                                None
                            }
                        })
                        .collect();
                    let remaining_failures: Vec<FailureEvent> = timeline[index..]
                        .iter()
                        .filter_map(|(_, item)| {
                            if let TimelineItem::Failure(failure) = item {
                                Some(failure.clone())
                            } else {
                                None
                            }
                        })
                        .collect();
                    let mut checkpoint = RepairCheckpoint {
                        execution_id: execution_id.to_string(),
                        group_id: explicit_group.map(|g| g.group_id.clone()).unwrap_or_default(),
                        checkpoint_sequence: 1,
                        at_s: clock.now_s(),
                        asset_statuses: asset_status_snapshot(&current.assets),
                        inventory: inventory.clone(),
                        completed_tasks,
                        unfinished_tasks,
                        selected_resources,
                        active_dependencies: repair.execution_dependencies.iter().cloned().collect(),
                        applied_failures,
                        pending_failures: pending.clone(),
                        remaining_failures,
                        group_status: "INTERRUPTED".into(),
                        restoration_target: repair.restoration_targets.iter().next().cloned().unwrap_or_default(),
                        event_sequence: event_sequence + 1,
                        event_batch_sequence: event_batch + 1,
                        remaining_horizon_s: clock.remaining_s(),
                        trace_history: interrupted_trace.clone(),
                        completed_plans: repair
                            .task_resources
                            .iter()
                            .enumerate()
                            .filter(|(index, _)| repair.completed_tasks.contains(index))
                            .filter_map(|(index, _)| repair.plans.get(index).cloned())
                            .collect(),
                        completed_planning_base: repair.planning_base.clone(),
                        source_environment: "canonical-v11".into(),
                        state_hash: String::new(),
                        predecessor_hash: String::new(),
                    };
                    checkpoint.state_hash = checkpoint.compute_state_hash();
                    **slot = Some(checkpoint);
                    event_sequence += 1;
                    event_batch += 1;
                    let assets = asset_status_snapshot(&current.assets);
                    record_event(
                        event_sequence,
                        event_batch,
                        clock.now_s(),
                        EventKind::CheckpointCreated,
                        EventOrigin::Endogenous,
                        Some("checkpoint".into()),
                        EventStatus::Committed,
                        &assets,
                        &assets,
                        &inventory,
                        &inventory,
                        Some(running.as_ref().expect("running repair")),
                    );
                }
                return Ok(EngineOutcome {
                    termination: EngineTermination::Blocked(format!(
                        "repair interrupted at {:.0}s: {} failed",
                        clock.now_s(),
                        hit.asset
                    )),
                    survived_s: clock.now_s(),
                    trace: interrupted_trace,
                    final_inventory: inventory,
                });
            }
        }
        pending.extend(repairable);
        observe_due(
            clock.now_s(),
            ObservationPhase::After,
            &inventory,
            running.as_ref(),
            &trace,
            &interval_flow,
            &asset_status_snapshot(&current.assets),
        );
    }

    let tail = clock.advance_to(options.horizon_s);
    match apply_energy_interval_ledger(&current, &mut inventory, tail, options.battery, options.load_kw) {
        Ok(flow) => {
            interval_flow.generated_j += flow.generated_j;
            interval_flow.load_j += flow.load_j;
            interval_flow.curtailed_j += flow.curtailed_j;
        }
        Err(offset) => {
            return Ok(EngineOutcome {
                termination: EngineTermination::Blocked("energy_depleted".into()),
                survived_s: clock.now_s() - tail + offset,
                trace,
                final_inventory: inventory,
            });
        }
    }
    let flow = std::mem::take(&mut interval_flow);
    let horizon_assets = asset_status_snapshot(&current.assets);
    event_sequence += 1;
    event_batch += 1;
    record_event(
        event_sequence,
        event_batch,
        clock.now_s(),
        EventKind::HorizonReached,
        EventOrigin::Endogenous,
        None,
        EventStatus::Committed,
        &horizon_assets,
        &horizon_assets,
        &inventory,
        &inventory,
        running.as_ref(),
    );
    trace.push(EngineTrace {
        at_s: clock.now_s(),
        kind: "horizon".into(),
        inventory_before: inventory.clone(),
        inventory_after: inventory.clone(),
        plans: Vec::new(),
        resources: Vec::new(),
        witness: Vec::new(),
        energy_generated_j: flow.generated_j,
        energy_resupply_j: 0.0,
        energy_load_j: flow.load_j,
        energy_repair_j: 0.0,
        energy_curtailed_j: flow.curtailed_j,
    });
    observe_due(
        clock.now_s(),
        ObservationPhase::After,
        &inventory,
        running.as_ref(),
        &trace,
        &interval_flow,
        &asset_status_snapshot(&current.assets),
    );
    Ok(EngineOutcome {
        termination: EngineTermination::CompletedHorizon,
        survived_s: clock.now_s(),
        trace,
        final_inventory: inventory,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::survival::simulate_multi_service_longevity_event_driven;

    fn base08() -> Model {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        Model::load(&root.join("scenarios/base-08/model.toml")).unwrap()
    }

    fn options<'a>(failures: Vec<FailureEvent>, horizon_s: f64) -> EngineOptions<'a> {
        EngineOptions {
            services: vec!["potable-water", "workshop-operability"],
            horizon_s,
            failures,
            resupplies: Vec::new(),
            resources: vec!["robot-m1".into(), "lathe-1".into()],
            alternative_pool: vec!["robot-m1".into()],
            battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
            load_kw: 0.0,
        }
    }

    #[test]
    fn canonical_matches_event_driven_on_a_sparse_scenario() {
        let mut model = base08();
        for lot in &mut model.lots {
            if lot.resource == "sleeve-p1" {
                lot.si = 10.0;
            }
        }
        model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap().duration_s =
            Some(5.0 * 86_400.0);
        let events: Vec<FailureEvent> = [10.0, 20.0]
            .into_iter()
            .map(|day| FailureEvent { at_s: day * 86_400.0, asset: "pump-p1".into() })
            .collect();
        let outcome = run_engine(&model, &options(events.clone(), 30.0 * 86_400.0)).unwrap();
        let legacy = simulate_multi_service_longevity_event_driven(
            &model,
            &["potable-water", "workshop-operability"],
            30.0 * 86_400.0,
            &events,
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
            0.0,
        )
        .unwrap();
        assert_eq!(outcome.survived_s, legacy.survived_s, "{:?} {:?}", outcome.termination, outcome.trace);
        assert_eq!(outcome.termination, EngineTermination::CompletedHorizon);
    }

    #[test]
    fn failure_during_initial_repair_takes_effect_on_time() {
        let mut model = base08();
        for lot in &mut model.lots {
            if lot.resource == "sleeve-p1" {
                lot.si = 0.0;
            }
            if lot.resource == "grease" {
                lot.si = 0.15;
            }
        }
        model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap().duration_s =
            Some(5.0 * 86_400.0);
        let outcome = run_engine(
            &model,
            &options(vec![FailureEvent { at_s: 2.0 * 86_400.0, asset: "pump-p1".into() }], 30.0 * 86_400.0),
        )
        .unwrap();
        assert!((outcome.survived_s - 2.0 * 86_400.0).abs() < 1.0, "{:?}", outcome.termination);
        assert!(
            matches!(outcome.termination, EngineTermination::Blocked(ref reason) if reason.contains("interrupted"))
        );
    }

    #[test]
    fn pv_failure_during_repair_stops_generation_from_that_time() {
        let mut model = base08();
        for lot in &mut model.lots {
            if lot.resource == "sleeve-p1" {
                lot.si = 1.0;
            }
        }
        model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap().duration_s =
            Some(5.0 * 3600.0);
        model.assets.get_mut("pump-p1").unwrap().status = AssetStatus::Failed;
        let control = run_engine(&model, &options(Vec::new(), 5.0 * 3600.0)).unwrap();
        let with_failure =
            run_engine(&model, &options(vec![FailureEvent { at_s: 2.0 * 3600.0, asset: "pv-1".into() }], 5.0 * 3600.0))
                .unwrap();
        assert!(with_failure.final_inventory["electric-energy"] < control.final_inventory["electric-energy"]);
        assert!(with_failure.trace.iter().any(|event| event.kind == "failure:pv-1" && event.at_s == 2.0 * 3600.0));
    }

    #[test]
    fn repair_completing_exactly_at_horizon_is_completed() {
        let mut model = base08();
        model.assets.get_mut("pump-p1").unwrap().status = AssetStatus::Available;
        let outcome =
            run_engine(&model, &options(vec![FailureEvent { at_s: 0.0, asset: "pump-p1".into() }], 3.0 * 3600.0))
                .unwrap();
        assert_eq!(outcome.termination, EngineTermination::CompletedHorizon);
        assert!(outcome
            .trace
            .iter()
            .any(|event| event.kind.starts_with("repair_completed") && event.at_s == 3.0 * 3600.0));
    }

    fn two_robot_options<'a>(failures: Vec<FailureEvent>, pool: Vec<String>) -> EngineOptions<'a> {
        EngineOptions {
            services: vec!["potable-water", "workshop-operability"],
            horizon_s: 5.0 * 3600.0,
            failures,
            resupplies: Vec::new(),
            resources: pool.clone(),
            alternative_pool: pool,
            battery: None,
            load_kw: 0.0,
        }
    }

    fn long_repair_model() -> Model {
        let mut model = base08();
        for lot in &mut model.lots {
            if lot.resource == "sleeve-p1" {
                lot.si = 1.0;
            }
        }
        model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap().duration_s =
            Some(5.0 * 3600.0);
        model.assets.get_mut("pump-p1").unwrap().status = AssetStatus::Failed;
        let mut second = model.assets.get("robot-m1").unwrap().clone();
        second.id = "robot-m2".into();
        model.assets.insert("robot-m2".into(), second);
        model
    }

    #[test]
    fn failure_of_the_selected_robot_interrupts_the_repair() {
        let model = long_repair_model();
        let pool = vec!["robot-m2".into(), "robot-m1".into()];
        let outcome = run_engine(
            &model,
            &two_robot_options(vec![FailureEvent { at_s: 2.0 * 3600.0, asset: "robot-m2".into() }], pool),
        )
        .unwrap();
        assert!(
            matches!(outcome.termination, EngineTermination::Blocked(ref reason) if reason.contains("interrupted"))
        );
        assert!(!outcome.trace.iter().any(|event| event.kind.starts_with("repair_completed")));
    }

    #[test]
    fn failure_of_an_unselected_robot_does_not_interrupt() {
        let model = long_repair_model();
        let pool = vec!["robot-m2".into(), "robot-m1".into()];
        let outcome = run_engine(
            &model,
            &two_robot_options(vec![FailureEvent { at_s: 2.0 * 3600.0, asset: "robot-m1".into() }], pool),
        )
        .unwrap();
        let completed = outcome.trace.iter().find(|event| event.kind.starts_with("repair_completed")).unwrap();
        assert!(completed.resources.contains(&"robot-m2".into()), "{:?}", completed.resources);
    }

    #[test]
    fn failure_of_an_upstream_dependency_interrupts_the_repair() {
        let model = long_repair_model();
        let outcome = run_engine(
            &model,
            &two_robot_options(
                vec![FailureEvent { at_s: 2.0 * 3600.0, asset: "bus-1".into() }],
                vec!["robot-m1".into()],
            ),
        )
        .unwrap();
        assert!(
            matches!(outcome.termination, EngineTermination::Blocked(ref reason) if reason.contains("interrupted"))
        );
    }

    #[test]
    fn failure_of_an_unselected_capability_provider_does_not_interrupt() {
        let mut model = long_repair_model();
        model.providers.entry("sleeve-replacement".into()).or_default().push("robot-m2".into());
        let pool = vec!["robot-m2".into(), "robot-m1".into()];
        let outcome = run_engine(
            &model,
            &two_robot_options(vec![FailureEvent { at_s: 2.0 * 3600.0, asset: "robot-m1".into() }], pool),
        )
        .unwrap();
        assert!(outcome.trace.iter().any(|event| event.kind.starts_with("repair_completed")));
    }

    #[test]
    fn failure_of_a_required_tool_interrupts_the_repair() {
        let mut model = long_repair_model();
        let mut tool = model.assets.get("robot-m1").unwrap().clone();
        tool.id = "tool-t1".into();
        tool.provides.clear();
        tool.requires.clear();
        tool.requires_any.clear();
        model.assets.insert("tool-t1".into(), tool);
        model.processes.iter_mut().find(|process| process.id == "replace-sleeve-p1").unwrap().occupies =
            vec!["tool-t1".into()];
        let outcome = run_engine(
            &model,
            &two_robot_options(
                vec![FailureEvent { at_s: 2.0 * 3600.0, asset: "tool-t1".into() }],
                vec!["robot-m1".into()],
            ),
        )
        .unwrap();
        assert!(
            matches!(outcome.termination, EngineTermination::Blocked(ref reason) if reason.contains("interrupted"))
        );
    }

    fn set_energy(model: &mut Model, kwh: f64) {
        use poie_ontology::model::{Lot, LotQuality};
        model.lots.retain(|lot| lot.resource != "electric-energy");
        model.lots.push(Lot {
            id: "test-energy".into(),
            resource: "electric-energy".into(),
            si: kwh * 3_600_000.0,
            quality: LotQuality::Accepted,
        });
    }

    #[test]
    fn rejects_initial_energy_over_battery_capacity() {
        let mut model = base08();
        set_energy(&mut model, 10.0);
        let mut opts = options(Vec::new(), 3600.0);
        opts.battery = Some(BatteryLimits { capacity_kwh: 1.0, charge_kw: 10.0 });
        assert!(run_engine(&model, &opts).is_err());
    }

    #[test]
    fn resupply_energy_is_capped_by_battery_capacity() {
        let mut model = base08();
        set_energy(&mut model, 0.5);
        model.assets.get_mut("pv-1").unwrap().status = AssetStatus::Failed;
        let mut opts = options(Vec::new(), 2.0 * 3600.0);
        opts.battery = Some(BatteryLimits { capacity_kwh: 1.0, charge_kw: 10.0 });
        opts.resupplies = vec![Resupply { at_s: 3600.0, resource: "electric-energy".into(), si: 5.0 * 3_600_000.0 }];
        let outcome = run_engine(&model, &opts).unwrap();
        assert!(outcome.final_inventory["electric-energy"] <= 3_600_000.0 + 1.0);
    }

    #[test]
    fn load_depletes_energy_at_the_correct_time() {
        let mut model = base08();
        set_energy(&mut model, 1.0);
        model.assets.get_mut("pv-1").unwrap().status = AssetStatus::Failed;
        let mut opts = options(Vec::new(), 5.0 * 3600.0);
        opts.load_kw = 1.0;
        let outcome = run_engine(&model, &opts).unwrap();
        assert!((outcome.survived_s - 3600.0).abs() < 1.0, "{}", outcome.survived_s);
        assert!(matches!(outcome.termination, EngineTermination::Blocked(ref reason) if reason == "energy_depleted"));
    }

    #[test]
    fn load_is_charged_over_the_repair_window() {
        let mut model = base08();
        set_energy(&mut model, 10.0);
        model.assets.get_mut("pv-1").unwrap().status = AssetStatus::Failed;
        model.assets.get_mut("pump-p1").unwrap().status = AssetStatus::Failed;
        for lot in &mut model.lots {
            if lot.resource == "sleeve-p1" {
                lot.si = 1.0;
            }
        }
        let mut opts = options(Vec::new(), 5.0 * 3600.0);
        opts.load_kw = 1.0;
        let outcome = run_engine(&model, &opts).unwrap();
        let expected = (10.0 - 5.0 - 0.5) * 3_600_000.0;
        assert!((outcome.final_inventory["electric-energy"] - expected).abs() < 1.0, "{:?}", outcome.final_inventory);
    }

    #[test]
    fn resupply_exactly_at_horizon_is_not_applied() {
        let mut model = base08();
        set_energy(&mut model, 1.0);
        model.assets.get_mut("pv-1").unwrap().status = AssetStatus::Failed;
        let mut opts = options(Vec::new(), 3600.0);
        opts.battery = Some(BatteryLimits { capacity_kwh: 10.0, charge_kw: 10.0 });
        opts.resupplies = vec![Resupply { at_s: 3600.0, resource: "electric-energy".into(), si: 2.0 * 3_600_000.0 }];
        let outcome = run_engine(&model, &opts).unwrap();
        assert!(!outcome.trace.iter().any(|event| event.kind.starts_with("resupply:")));
        assert!((outcome.final_inventory["electric-energy"] - 3_600_000.0).abs() < 1.0);
    }

    #[test]
    fn joint_energy_ledger_conserves_every_flow() {
        let mut model = base08();
        set_energy(&mut model, 6.0);
        model.assets.get_mut("pump-p1").unwrap().status = AssetStatus::Failed;
        for lot in &mut model.lots {
            if lot.resource == "sleeve-p1" {
                lot.si = 1.0;
            }
        }
        let mut opts = options(Vec::new(), 3.0 * 3600.0);
        opts.battery = Some(BatteryLimits { capacity_kwh: 10.0, charge_kw: 10.0 });
        opts.load_kw = 1.0;
        opts.resupplies = vec![Resupply { at_s: 3600.0, resource: "electric-energy".into(), si: 2.0 * 3_600_000.0 }];
        let outcome = run_engine(&model, &opts).unwrap();
        let generated: f64 = outcome.trace.iter().map(|event| event.energy_generated_j).sum();
        let resupply: f64 = outcome.trace.iter().map(|event| event.energy_resupply_j).sum();
        let load: f64 = outcome.trace.iter().map(|event| event.energy_load_j).sum();
        let repair: f64 = outcome.trace.iter().map(|event| event.energy_repair_j).sum();
        let curtailed: f64 = outcome.trace.iter().map(|event| event.energy_curtailed_j).sum();
        let initial = 6.0 * 3_600_000.0;
        let final_j = outcome.final_inventory["electric-energy"];
        assert!((final_j - 9.5 * 3_600_000.0).abs() < 1.0, "{}", final_j / 3_600_000.0);
        assert!((generated - 7.5 * 3_600_000.0).abs() < 1.0);
        assert!((resupply - 2.0 * 3_600_000.0).abs() < 1.0);
        assert!((load - 3.0 * 3_600_000.0).abs() < 1.0);
        assert!((repair - 0.5 * 3_600_000.0).abs() < 1.0);
        assert!((curtailed - 2.5 * 3_600_000.0).abs() < 1.0);
        let conserved = initial + generated + resupply - load - repair - curtailed;
        assert!((conserved - final_j).abs() < 1.0, "{conserved} vs {final_j}");
    }

    #[test]
    fn resupply_cannot_be_used_before_it_arrives() {
        let mut model = base08();
        model.assets.get_mut("pump-p1").unwrap().status = AssetStatus::Failed;
        for lot in &mut model.lots {
            if lot.resource == "sleeve-p1" {
                lot.si = 1.0;
            }
            if lot.resource == "grease" {
                lot.si = 0.0;
            }
        }
        let mut opts = options(Vec::new(), 4.0 * 3600.0);
        opts.resupplies = vec![Resupply { at_s: 3600.0, resource: "grease".into(), si: 1.0 }];
        let outcome = run_engine(&model, &opts).unwrap();
        assert!(
            matches!(outcome.termination, EngineTermination::Blocked(ref reason) if reason.contains("grease")),
            "{:?}",
            outcome.termination
        );
    }

    #[test]
    fn redundant_service_alternative_suppresses_unnecessary_repair() {
        use poie_ontology::model::Asset;
        use std::collections::BTreeMap;

        let mut model = base08();
        model.assets.get_mut("pump-p1").unwrap().status = AssetStatus::Available;
        model.assets.insert(
            "pump-p2".into(),
            Asset {
                id: "pump-p2".into(),
                kind: "pump".into(),
                status: AssetStatus::Available,
                status_evidence: None,
                requires: Vec::new(),
                requires_any: Vec::new(),
                provides: Vec::new(),
                capability_verification: BTreeMap::new(),
                degraded: None,
            },
        );
        let service = model.services.get_mut("potable-water").unwrap();
        service.requires.retain(|asset| asset != "pump-p1");
        service.requires_any = vec!["pump-p1".into(), "pump-p2".into()];
        let outcome = run_engine(
            &model,
            &options(vec![FailureEvent { at_s: 10.0 * 86_400.0, asset: "pump-p1".into() }], 90.0 * 86_400.0),
        )
        .unwrap();
        assert_eq!(outcome.termination, EngineTermination::CompletedHorizon);
        assert!(!outcome.trace.iter().any(|event| event.kind.starts_with("repair_completed")));
    }

    #[test]
    fn zero_time_failure_is_processed_after_initial_repair() {
        let mut model = base08();
        model.assets.get_mut("pump-p1").unwrap().status = AssetStatus::Failed;
        let outcome =
            run_engine(&model, &options(vec![FailureEvent { at_s: 0.0, asset: "filter-1".into() }], 24.0 * 3600.0))
                .unwrap();
        assert!((outcome.survived_s - 3.0 * 3600.0).abs() < 1.0, "{:?}", outcome.termination);
        assert!(matches!(outcome.termination, EngineTermination::Blocked(ref reason) if reason.contains("filter-1")));
    }

    #[test]
    fn non_service_asset_failure_does_not_trigger_repair_or_block() {
        let mut model = base08();
        model.assets.get_mut("pump-p1").unwrap().status = AssetStatus::Available;
        let outcome =
            run_engine(&model, &options(vec![FailureEvent { at_s: 3600.0, asset: "charger-1".into() }], 2.0 * 3600.0))
                .unwrap();
        assert_eq!(outcome.termination, EngineTermination::CompletedHorizon);
        assert!(outcome.trace.iter().any(|event| event.kind == "failure:charger-1"));
        assert!(!outcome.trace.iter().any(|event| event.kind.starts_with("repair_completed")));
    }

    #[test]
    fn same_time_resupply_and_failure_both_apply_once() {
        let model = base08();
        let mut opts = options(vec![FailureEvent { at_s: 86_400.0, asset: "lathe-1".into() }], 2.0 * 86_400.0);
        opts.resupplies = vec![Resupply { at_s: 86_400.0, resource: "grease".into(), si: 0.5 }];
        let outcome = run_engine(&model, &opts).unwrap();
        let resupplies = outcome.trace.iter().filter(|event| event.kind == "resupply:grease").count();
        let failures = outcome.trace.iter().filter(|event| event.kind == "failure:lathe-1").count();
        assert_eq!(resupplies, 1, "{:?}", outcome.trace);
        assert_eq!(failures, 1, "{:?}", outcome.trace);
    }

    #[test]
    fn canonical_termination_is_typed() {
        let model = base08();
        let outcome =
            run_engine(&model, &options(vec![FailureEvent { at_s: 0.0, asset: "filter-1".into() }], 86_400.0)).unwrap();
        assert!(matches!(outcome.termination, EngineTermination::Blocked(_)));
    }
    #[test]
    fn native_asset_status_before_after_restoration() {
        let model = base08();
        let opts = options(Vec::new(), 20.0 * 3600.0);
        let direct = run_engine(&model, &opts).unwrap();
        let observed = run_engine_observed(
            &model,
            &opts,
            &[
                ObservationRequest { at_s: 10800.0, phase: ObservationPhase::Before },
                ObservationRequest { at_s: 10800.0, phase: ObservationPhase::After },
            ],
        )
        .unwrap();
        assert_eq!(direct, observed.outcome);
        let before = &observed.observations[0];
        let after = &observed.observations[1];
        assert_eq!(before.asset_statuses["pump-p1"], AssetStatus::Degraded);
        assert_eq!(after.asset_statuses["pump-p1"], AssetStatus::Available);
        assert_eq!(before.asset_status_source, "NATIVE_CANONICAL_RUNTIME");
        assert!(before.running_repair.is_some());
        assert!(after.running_repair.is_none());
    }

    #[test]
    fn native_asset_status_after_interruption() {
        let model = base08();
        let opts = options(vec![FailureEvent { at_s: 7200.0, asset: "robot-m1".into() }], 20.0 * 3600.0);
        let direct = run_engine(&model, &opts).unwrap();
        let observed =
            run_engine_observed(&model, &opts, &[ObservationRequest { at_s: 7200.0, phase: ObservationPhase::After }])
                .unwrap();
        assert_eq!(direct, observed.outcome);
        let after = &observed.observations[0];
        assert_eq!(after.status, ObservationStatus::Observed);
        assert_eq!(after.asset_statuses["robot-m1"], AssetStatus::Failed);
        assert_eq!(after.asset_statuses["pump-p1"], AssetStatus::Degraded);
        assert_eq!(after.asset_status_source, "NATIVE_CANONICAL_RUNTIME");
        assert!(after.running_repair.is_some());
    }

    #[test]
    fn native_observation_context_is_restored_between_runs() {
        let model = base08();
        let opts = options(Vec::new(), 20.0 * 3600.0);
        let first =
            run_engine_observed(&model, &opts, &[ObservationRequest { at_s: 3600.0, phase: ObservationPhase::After }])
                .unwrap();
        let second =
            run_engine_observed(&model, &opts, &[ObservationRequest { at_s: 3600.0, phase: ObservationPhase::After }])
                .unwrap();
        assert_eq!(first.observations[0].asset_statuses, second.observations[0].asset_statuses);
        let joined = std::thread::spawn(move || {
            let model = base08();
            let opts = options(Vec::new(), 20.0 * 3600.0);
            run_engine_observed(&model, &opts, &[ObservationRequest { at_s: 3600.0, phase: ObservationPhase::After }])
                .unwrap()
                .observations[0]
                .asset_statuses
                .clone()
        })
        .join()
        .unwrap();
        assert_eq!(first.observations[0].asset_statuses, joined);
    }

    #[test]
    fn native_observation_context_survives_error() {
        let model = base08();
        let bad = EngineOptions {
            services: vec!["missing-service"],
            horizon_s: 1.0,
            failures: Vec::new(),
            resupplies: Vec::new(),
            resources: Vec::new(),
            alternative_pool: Vec::new(),
            battery: None,
            load_kw: 0.0,
        };
        assert!(run_engine_observed(&model, &bad, &[]).is_err());
        let valid = options(Vec::new(), 20.0 * 3600.0);
        assert!(run_engine_observed(&model, &valid, &[]).is_ok());
    }
    #[test]
    fn nested_observation_is_rejected_and_outer_context_restored() {
        let collector = ObservationCollector { requests: Vec::new(), index: 0, observations: Vec::new() };
        let guard = ObservationGuard::install(collector).expect("install outer observation");
        let model = base08();
        let opts = options(Vec::new(), 20.0 * 3600.0);
        assert!(run_engine_observed(&model, &opts, &[]).is_err());
        drop(guard);
        assert!(run_engine_observed(&model, &opts, &[]).is_ok());
    }
    #[test]
    fn event_resolved_completion_and_failure_are_distinct_at_same_timestamp() {
        let model = base08();
        let opts = options(vec![FailureEvent { at_s: 10800.0, asset: "robot-m1".into() }], 20.0 * 3600.0);
        let direct = run_engine(&model, &opts).unwrap();
        let observed = run_engine_event_observed(&model, &opts).unwrap();
        assert_eq!(direct, observed.outcome);
        let completion = observed.events.iter().find(|event| event.kind == EventKind::RepairCompleted).unwrap();
        let failure = observed.events.iter().find(|event| event.kind == EventKind::AssetFailure).unwrap();
        assert_eq!(completion.at_s, 10800.0);
        assert_eq!(failure.at_s, 10800.0);
        assert!(completion.event_sequence < failure.event_sequence);
        assert_ne!(completion.event_id, failure.event_id);
        assert_eq!(completion.asset_statuses_before["pump-p1"], AssetStatus::Degraded);
        assert_eq!(completion.asset_statuses_after["pump-p1"], AssetStatus::Available);
        assert_eq!(failure.asset_statuses_before["robot-m1"], AssetStatus::Available);
        assert_eq!(failure.asset_statuses_after["robot-m1"], AssetStatus::Failed);
        let ids: std::collections::BTreeSet<&str> =
            observed.events.iter().map(|event| event.event_id.as_str()).collect();
        assert_eq!(ids.len(), observed.events.len());
        assert_eq!(observed.events.last().unwrap().kind, EventKind::HorizonReached);
    }
    #[test]
    fn nested_event_observation_is_rejected_and_context_restored() {
        let guard = EventGuard::install().expect("install outer event collector");
        let model = base08();
        let opts = options(Vec::new(), 20.0 * 3600.0);
        assert!(run_engine_event_observed(&model, &opts).is_err());
        drop(guard);
        assert!(run_engine_event_observed(&model, &opts).is_ok());
    }

    #[test]
    fn event_identity_is_stable_across_threads() {
        let joined = std::thread::spawn(|| {
            let model = base08();
            let opts = options(vec![FailureEvent { at_s: 10800.0, asset: "robot-m1".into() }], 20.0 * 3600.0);
            let observed = run_engine_event_observed(&model, &opts).unwrap();
            observed.events.iter().map(|event| event.event_id.clone()).collect::<Vec<_>>()
        })
        .join()
        .unwrap();
        let model = base08();
        let opts = options(vec![FailureEvent { at_s: 10800.0, asset: "robot-m1".into() }], 20.0 * 3600.0);
        let observed = run_engine_event_observed(&model, &opts).unwrap();
        let local = observed.events.iter().map(|event| event.event_id.clone()).collect::<Vec<_>>();
        assert_eq!(joined, local);
    }
    #[test]
    fn event_observation_exports_resource_witness_fields() {
        let model = base08();
        let opts = options(Vec::new(), 20.0 * 3600.0);
        let observed = run_engine_event_observed(&model, &opts).unwrap();
        let initial = observed.events.first().expect("initial event");
        let repair = initial.running_repair.as_ref().expect("running repair");
        assert!(!repair.resources.is_empty());
        assert!(repair.restoration_targets.contains(&"pump-p1".to_string()));
        assert!(!repair.execution_dependencies.is_empty());
        assert!(!repair.witness.is_empty());
    }
}
