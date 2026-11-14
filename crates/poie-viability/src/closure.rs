// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S1: recursive capability-closure analysis.
//!
//! This module answers a narrow, checkable question over one frozen model:
//! from the model's initial usable assets and accepted inventory, can a target
//! asset be restored within a time horizon, using only local restorers,
//! capabilities, occupied assets and producible resources?
//!
//! The analysis is deliberately serial and monotone. It does not schedule
//! concurrent work, does not assume a process can run without a verified or
//! modelled duration, and consumes inventory as it searches. A cycle in the
//! repair dependency graph is reported as a cycle rather than guessed away.
//! Consequently a positive result is a model-level existence result under
//! those assumptions, not a physical safety or timing claim.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use poie_ontology::model::{AssetStatus, LotQuality, Model, Process, Verification};

const EPSILON: f64 = 1e-9;

fn apply_process_outputs(process: &Process, inventory: &mut BTreeMap<String, f64>) {
    for output in &process.outputs {
        let fraction = process.yield_fraction_for(&output.resource);
        *inventory.entry(output.resource.clone()).or_insert(0.0) += output.si * fraction;
        if let Some(spec) = &process.process_yield {
            if spec.output == output.resource && fraction < 1.0 {
                *inventory.entry(spec.reject.clone()).or_insert(0.0) += output.si * (1.0 - fraction);
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub enum StepAction {
    Restore(String),
    Produce(String),
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Step {
    pub process: String,
    pub action: StepAction,
    pub duration_s: f64,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Plan {
    pub steps: Vec<Step>,
    pub total_duration_s: f64,
    pub remaining_inventory: BTreeMap<String, f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BlockReason {
    UnknownAsset(String),
    NoRestorer(String),
    MissingResource { resource: String, needed: f64, available: f64 },
    MissingCapability(String),
    Cycle { kind: &'static str, id: String },
    UnknownDuration(String),
    HorizonExceeded { needed_s: f64, horizon_s: f64 },
    UnverifiedProcess { process: String, required: Verification, actual: Verification },
    UnverifiedCapability { capability: String, required: Verification, actual: Verification },
}

impl fmt::Display for BlockReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockReason::UnknownAsset(asset) => write!(f, "unknown asset {asset}"),
            BlockReason::NoRestorer(asset) => write!(f, "no restorer for {asset}"),
            BlockReason::MissingResource { resource, needed, available } => {
                write!(f, "missing resource {resource}: need {needed}, have {available}")
            }
            BlockReason::MissingCapability(capability) => write!(f, "missing capability {capability}"),
            BlockReason::Cycle { kind, id } => write!(f, "cyclic {kind} dependency at {id}"),
            BlockReason::UnknownDuration(process) => write!(f, "unknown duration for {process}"),
            BlockReason::HorizonExceeded { needed_s, horizon_s } => {
                write!(f, "horizon exceeded: need {needed_s}s, horizon {horizon_s}s")
            }
            BlockReason::UnverifiedProcess { process, required, actual } => {
                write!(f, "unverified process {process}: {actual:?} < {required:?}")
            }
            BlockReason::UnverifiedCapability { capability, required, actual } => {
                write!(f, "unverified capability {capability}: {actual:?} < {required:?}")
            }
        }
    }
}

impl BlockReason {
    fn score(&self) -> u8 {
        match self {
            BlockReason::Cycle { .. } => 0,
            BlockReason::MissingResource { .. } => 1,
            BlockReason::MissingCapability(_) => 2,
            BlockReason::NoRestorer(_) => 3,
            BlockReason::UnverifiedProcess { .. } | BlockReason::UnverifiedCapability { .. } => 4,
            BlockReason::UnknownDuration(_) => 5,
            BlockReason::HorizonExceeded { .. } => 6,
            BlockReason::UnknownAsset(_) => 7,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Blocked {
    pub reasons: Vec<BlockReason>,
    pub horizon_s: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClosureOutcome {
    Restorable(Plan),
    Blocked(Blocked),
}

impl ClosureOutcome {
    pub fn is_restorable(&self) -> bool {
        matches!(self, ClosureOutcome::Restorable(_))
    }
}

#[derive(Clone, Debug)]
struct State {
    usable: BTreeSet<String>,
    inventory: BTreeMap<String, f64>,
    steps: Vec<Step>,
    total_s: f64,
}

impl State {
    fn initial(model: &Model) -> State {
        let mut usable = BTreeSet::new();
        for asset in model.assets.keys() {
            if initially_usable(model, asset, &mut BTreeSet::new()) {
                usable.insert(asset.clone());
            }
        }
        let mut inventory = BTreeMap::new();
        for lot in &model.lots {
            if lot.quality == LotQuality::Accepted && lot.si > 0.0 {
                *inventory.entry(lot.resource.clone()).or_insert(0.0) += lot.si;
            }
        }
        State { usable, inventory, steps: Vec::new(), total_s: 0.0 }
    }
}

fn initially_usable(model: &Model, asset_id: &str, seen: &mut BTreeSet<String>) -> bool {
    if !seen.insert(asset_id.to_string()) {
        return false;
    }
    let Some(asset) = model.assets.get(asset_id) else {
        return false;
    };
    if asset.status != AssetStatus::Available {
        return false;
    }
    if !asset.requires.iter().all(|dep| initially_usable(model, dep, seen)) {
        return false;
    }
    if !asset.requires_any.is_empty() && !asset.requires_any.iter().any(|dep| initially_usable(model, dep, seen)) {
        return false;
    }
    true
}

#[derive(Clone, Debug)]
struct Failure {
    reasons: Vec<BlockReason>,
}

impl Failure {
    fn one(reason: BlockReason) -> Failure {
        Failure { reasons: vec![reason] }
    }

    fn better(self, other: Failure) -> Failure {
        let self_score = self.reasons.iter().map(BlockReason::score).min().unwrap_or(u8::MAX);
        let other_score = other.reasons.iter().map(BlockReason::score).min().unwrap_or(u8::MAX);
        if other_score < self_score {
            other
        } else {
            self
        }
    }
}

pub fn analyze(model: &Model, target: &str, horizon_s: f64) -> ClosureOutcome {
    analyze_with_min_verification(model, target, horizon_s, Verification::Modeled)
}

pub fn analyze_with_min_verification(
    model: &Model,
    target: &str,
    horizon_s: f64,
    min_verification: Verification,
) -> ClosureOutcome {
    if !model.assets.contains_key(target) {
        return ClosureOutcome::Blocked(Blocked {
            reasons: vec![BlockReason::UnknownAsset(target.to_string())],
            horizon_s,
        });
    }
    let search = Search { model, horizon_s, min_verification };
    let state = State::initial(model);
    let mut assets = Vec::new();
    let mut resources = Vec::new();
    let mut processes = Vec::new();
    match search.solve_asset(target, state, &mut assets, &mut resources, &mut processes) {
        Ok(state) => ClosureOutcome::Restorable(Plan {
            steps: state.steps,
            total_duration_s: state.total_s,
            remaining_inventory: state.inventory,
        }),
        Err(failure) => ClosureOutcome::Blocked(Blocked { reasons: failure.reasons, horizon_s }),
    }
}

struct Search<'a> {
    model: &'a Model,
    horizon_s: f64,
    min_verification: Verification,
}

impl Search<'_> {
    fn solve_asset(
        &self,
        asset_id: &str,
        state: State,
        stack_assets: &mut Vec<String>,
        stack_resources: &mut Vec<String>,
        stack_processes: &mut Vec<String>,
    ) -> Result<State, Failure> {
        if state.usable.contains(asset_id) {
            return Ok(state);
        }
        if stack_assets.iter().any(|item| item == asset_id) {
            return Err(Failure::one(BlockReason::Cycle { kind: "asset", id: asset_id.to_string() }));
        }
        let Some(asset) = self.model.assets.get(asset_id) else {
            return Err(Failure::one(BlockReason::UnknownAsset(asset_id.to_string())));
        };
        let restorers = self.model.restorers.get(asset_id).cloned().unwrap_or_default();
        if restorers.is_empty() {
            // A failed asset with no restorer can still expose the deeper
            // reason through its support chain. Prefer a cycle in that chain
            // over the shallower "no restorer" message.
            match self.ensure_supports(asset, state.clone(), stack_assets, stack_resources, stack_processes) {
                Err(failure) if failure.reasons.iter().any(|reason| matches!(reason, BlockReason::Cycle { .. })) => {
                    return Err(failure);
                }
                _ => return Err(Failure::one(BlockReason::NoRestorer(asset_id.to_string()))),
            }
        }
        let mut best: Option<Failure> = None;
        for index in restorers {
            let process = &self.model.processes[index];
            if process.verification < self.min_verification {
                let failure = Failure::one(BlockReason::UnverifiedProcess {
                    process: process.id.clone(),
                    required: self.min_verification,
                    actual: process.verification,
                });
                best = Some(match best {
                    Some(current) => current.better(failure),
                    None => failure,
                });
                continue;
            }
            stack_assets.push(asset_id.to_string());
            let result = self.run_process(
                process,
                state.clone(),
                stack_assets,
                stack_resources,
                stack_processes,
                StepAction::Restore(asset_id.to_string()),
            );
            stack_assets.pop();
            match result {
                Ok(next) => match self.ensure_supports(asset, next, stack_assets, stack_resources, stack_processes) {
                    Ok(mut supported) => {
                        supported.usable.insert(asset_id.to_string());
                        return Ok(supported);
                    }
                    Err(failure) => {
                        best = Some(match best {
                            Some(current) => current.better(failure),
                            None => failure,
                        });
                    }
                },
                Err(failure) => {
                    best = Some(match best {
                        Some(current) => current.better(failure),
                        None => failure,
                    });
                }
            }
        }
        Err(best.unwrap_or_else(|| Failure::one(BlockReason::NoRestorer(asset_id.to_string()))))
    }

    fn ensure_supports(
        &self,
        asset: &poie_ontology::model::Asset,
        mut state: State,
        stack_assets: &mut Vec<String>,
        stack_resources: &mut Vec<String>,
        stack_processes: &mut Vec<String>,
    ) -> Result<State, Failure> {
        for dependency in &asset.requires {
            state = self.solve_asset(dependency, state, stack_assets, stack_resources, stack_processes)?;
        }
        if !asset.requires_any.is_empty() {
            let mut best: Option<Failure> = None;
            for dependency in &asset.requires_any {
                match self.solve_asset(dependency, state.clone(), stack_assets, stack_resources, stack_processes) {
                    Ok(next) => {
                        state = next;
                        best = None;
                        break;
                    }
                    Err(failure) => {
                        best = Some(match best {
                            Some(current) => current.better(failure),
                            None => failure,
                        });
                    }
                }
            }
            if let Some(failure) = best {
                return Err(failure);
            }
        }
        Ok(state)
    }

    fn run_process(
        &self,
        process: &Process,
        mut state: State,
        stack_assets: &mut Vec<String>,
        stack_resources: &mut Vec<String>,
        stack_processes: &mut Vec<String>,
        action: StepAction,
    ) -> Result<State, Failure> {
        if stack_processes.iter().any(|id| id == &process.id) {
            return Err(Failure::one(BlockReason::Cycle { kind: "process", id: process.id.clone() }));
        }
        let Some(duration_s) = process.duration_s else {
            return Err(Failure::one(BlockReason::UnknownDuration(process.id.clone())));
        };
        if state.total_s + duration_s > self.horizon_s + EPSILON {
            return Err(Failure::one(BlockReason::HorizonExceeded {
                needed_s: state.total_s + duration_s,
                horizon_s: self.horizon_s,
            }));
        }
        stack_processes.push(process.id.clone());
        let result = (|| {
            for occupied in &process.occupies {
                state = self.solve_asset(occupied, state, stack_assets, stack_resources, stack_processes)?;
            }
            for capability in &process.requires_capabilities {
                state = self.ensure_capability(capability, state, stack_assets, stack_resources, stack_processes)?;
            }
            for input in &process.inputs {
                state = self.ensure_resource(
                    &input.resource,
                    input.si,
                    state,
                    stack_assets,
                    stack_resources,
                    stack_processes,
                )?;
            }
            apply_process_outputs(process, &mut state.inventory);
            state.total_s += duration_s;
            state.steps.push(Step { process: process.id.clone(), action, duration_s });
            Ok(state)
        })();
        stack_processes.pop();
        result
    }

    fn ensure_capability(
        &self,
        capability: &str,
        state: State,
        stack_assets: &mut Vec<String>,
        stack_resources: &mut Vec<String>,
        stack_processes: &mut Vec<String>,
    ) -> Result<State, Failure> {
        let providers = self.model.providers.get(capability).cloned().unwrap_or_default();
        if providers.is_empty() {
            return Err(Failure::one(BlockReason::MissingCapability(capability.to_string())));
        }
        let mut best: Option<Failure> = None;
        for provider in providers {
            if let Some(asset) = self.model.assets.get(&provider) {
                let actual = asset.capability_verification(capability);
                if actual < self.min_verification {
                    let failure = Failure::one(BlockReason::UnverifiedCapability {
                        capability: capability.to_string(),
                        required: self.min_verification,
                        actual,
                    });
                    best = Some(match best {
                        Some(current) => current.better(failure),
                        None => failure,
                    });
                    continue;
                }
            }
            match self.solve_asset(&provider, state.clone(), stack_assets, stack_resources, stack_processes) {
                Ok(next) => return Ok(next),
                Err(failure) => {
                    best = Some(match best {
                        Some(current) => current.better(failure),
                        None => failure,
                    });
                }
            }
        }
        Err(best.unwrap_or_else(|| Failure::one(BlockReason::MissingCapability(capability.to_string()))))
    }

    fn ensure_resource(
        &self,
        resource: &str,
        quantity: f64,
        mut state: State,
        stack_assets: &mut Vec<String>,
        stack_resources: &mut Vec<String>,
        stack_processes: &mut Vec<String>,
    ) -> Result<State, Failure> {
        let available = state.inventory.get(resource).copied().unwrap_or(0.0);
        if available + EPSILON >= quantity {
            let remaining = available - quantity;
            if remaining <= EPSILON {
                state.inventory.remove(resource);
            } else {
                state.inventory.insert(resource.to_string(), remaining);
            }
            return Ok(state);
        }
        if stack_resources.iter().any(|id| id == resource) {
            return Err(Failure::one(BlockReason::Cycle { kind: "resource", id: resource.to_string() }));
        }
        let producers = self.model.producers.get(resource).cloned().unwrap_or_default();
        if producers.is_empty() {
            return Err(Failure::one(BlockReason::MissingResource {
                resource: resource.to_string(),
                needed: quantity,
                available,
            }));
        }
        stack_resources.push(resource.to_string());
        let mut best: Option<Failure> = None;
        for index in producers {
            let process = &self.model.processes[index];
            if process.verification < self.min_verification {
                let failure = Failure::one(BlockReason::UnverifiedProcess {
                    process: process.id.clone(),
                    required: self.min_verification,
                    actual: process.verification,
                });
                best = Some(match best {
                    Some(current) => current.better(failure),
                    None => failure,
                });
                continue;
            }
            match self.run_process(
                process,
                state.clone(),
                stack_assets,
                stack_resources,
                stack_processes,
                StepAction::Produce(resource.to_string()),
            ) {
                Ok(mut next) => {
                    let mut produced = next.inventory.get(resource).copied().unwrap_or(0.0);
                    // A process with sub-unity yield may need several runs to
                    // cover one requirement. Repeat the run while it makes
                    // progress; the recursion and horizon checks bound the loop.
                    let mut runs = 1usize;
                    while produced + EPSILON < quantity && runs < 20 {
                        let before = produced;
                        match self.run_process(
                            process,
                            next.clone(),
                            stack_assets,
                            stack_resources,
                            stack_processes,
                            StepAction::Produce(resource.to_string()),
                        ) {
                            Ok(more) => {
                                next = more;
                                produced = next.inventory.get(resource).copied().unwrap_or(0.0);
                                if produced <= before + EPSILON {
                                    break;
                                }
                                runs += 1;
                            }
                            Err(_) => break,
                        }
                    }
                    if produced + EPSILON >= quantity {
                        let remaining = produced - quantity;
                        if remaining <= EPSILON {
                            next.inventory.remove(resource);
                        } else {
                            next.inventory.insert(resource.to_string(), remaining);
                        }
                        stack_resources.pop();
                        return Ok(next);
                    }
                    let failure = Failure::one(BlockReason::MissingResource {
                        resource: resource.to_string(),
                        needed: quantity,
                        available: produced,
                    });
                    best = Some(match best {
                        Some(current) => current.better(failure),
                        None => failure,
                    });
                }
                Err(failure) => {
                    best = Some(match best {
                        Some(current) => current.better(failure),
                        None => failure,
                    });
                }
            }
        }
        stack_resources.pop();
        Err(best.unwrap_or_else(|| {
            Failure::one(BlockReason::MissingResource { resource: resource.to_string(), needed: quantity, available })
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use poie_ontology::model::Flow;
    use std::path::Path;

    fn load(relative: &str) -> Model {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        Model::load(&root.join(relative)).expect("load model")
    }

    fn failed(mut model: Model, asset: &str) -> Model {
        model.assets.get_mut(asset).expect("asset").status = AssetStatus::Failed;
        model
    }

    #[test]
    fn base04_lathe_with_spare_is_restorable() {
        let model = failed(load("scenarios/base-04/model.toml"), "lathe-1");
        let ClosureOutcome::Restorable(plan) = analyze(&model, "lathe-1", 12.0 * 3600.0) else {
            panic!("expected restorable");
        };
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].process, "replace-lathe-nut");
        assert_eq!(plan.total_duration_s, 7200.0);
    }

    #[test]
    fn base01_lathe_without_spare_is_cycle_blocked() {
        let model = failed(load("scenarios/base-01/model.toml"), "lathe-1");
        let ClosureOutcome::Blocked(blocked) = analyze(&model, "lathe-1", 12.0 * 3600.0) else {
            panic!("expected blocked");
        };
        assert!(blocked.reasons.iter().any(|reason| matches!(
            reason,
            BlockReason::Cycle { kind: "asset", id } if id == "lathe-1"
        )));
    }

    #[test]
    fn pump_can_rebuild_a_missing_sleeve() {
        let mut model = load("scenarios/base-01/model.toml");
        model.lots.retain(|lot| lot.id != "lot-sleeve-01");
        let ClosureOutcome::Restorable(plan) = analyze(&model, "pump-p1", 12.0 * 3600.0) else {
            panic!("expected restorable");
        };
        assert_eq!(
            plan.steps.iter().map(|step| step.process.as_str()).collect::<Vec<_>>(),
            vec!["turn-sleeve-p1", "inspect-sleeve-p1", "replace-sleeve-p1"]
        );
        assert_eq!(plan.total_duration_s, 27_000.0);
    }

    #[test]
    fn missing_breaker_is_not_restorable() {
        let mut model = load("scenarios/base-01/model.toml");
        model.assets.get_mut("bus-1").expect("bus").status = AssetStatus::Failed;
        model.lots.retain(|lot| lot.id != "lot-breaker");
        let ClosureOutcome::Blocked(blocked) = analyze(&model, "bus-1", 12.0 * 3600.0) else {
            panic!("expected blocked");
        };
        assert!(blocked
            .reasons
            .iter()
            .any(|reason| matches!(reason, BlockReason::Cycle { kind: "asset", id } if id == "bus-1")));
    }

    #[test]
    fn pump_repair_exceeds_a_short_horizon() {
        let model = load("scenarios/base-04/model.toml");
        let ClosureOutcome::Blocked(blocked) = analyze(&model, "pump-p1", 2.0 * 3600.0) else {
            panic!("expected blocked");
        };
        assert!(blocked.reasons.iter().any(|reason| matches!(reason, BlockReason::HorizonExceeded { .. })));
    }

    #[test]
    fn unverified_capability_provider_is_blocked() {
        let mut model = failed(load("scenarios/base-04/model.toml"), "lathe-1");
        model
            .assets
            .get_mut("robot-m1")
            .expect("robot")
            .capability_verification
            .insert("lathe-repair".into(), Verification::Declared);
        let ClosureOutcome::Blocked(blocked) = analyze(&model, "lathe-1", 12.0 * 3600.0) else {
            panic!("expected blocked");
        };
        assert!(blocked.reasons.iter().any(|reason| matches!(
            reason,
            BlockReason::UnverifiedCapability { capability, .. } if capability == "lathe-repair"
        )));
    }

    #[test]
    fn unknown_target_is_blocked() {
        let model = load("scenarios/base-01/model.toml");
        let ClosureOutcome::Blocked(blocked) = analyze(&model, "does-not-exist", 3600.0) else {
            panic!("expected blocked");
        };
        assert_eq!(blocked.reasons, vec![BlockReason::UnknownAsset("does-not-exist".into())]);
    }
    #[test]
    fn process_yield_applies_good_fraction_and_reject() {
        let process = Process {
            id: "yielded".into(),
            inputs: Vec::new(),
            outputs: vec![Flow { resource: "good".into(), si: 10.0 }],
            occupies: Vec::new(),
            requires_capabilities: Vec::new(),
            restores: Vec::new(),
            duration_s: Some(60.0),
            verification: Verification::Modeled,
            asset_mass_change_kg: 0.0,
            mass_tolerance: 0.01,
            process_yield: Some(poie_ontology::model::YieldSpec {
                output: "good".into(),
                fraction: 0.8,
                reject: "reject".into(),
            }),
        };
        let mut inventory = BTreeMap::new();
        apply_process_outputs(&process, &mut inventory);
        assert!((inventory["good"] - 8.0).abs() < 1e-9);
        assert!((inventory["reject"] - 2.0).abs() < 1e-9);
    }
}
