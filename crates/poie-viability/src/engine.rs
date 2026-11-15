// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Canonical event execution kernel.
//!
//! One clock, one inventory ledger and one commit rule for initial repair,
//! failures, resupply and horizon. The kernel emits a typed termination, a
//! trace with before/after inventories, and the plan processes it applied.
//! Legacy `survival` entry points are not yet migrated; this is the target
//! implementation for S5-T.

use std::collections::{BTreeMap, BTreeSet};

use poie_ontology::model::{AssetStatus, Model, Service};

use crate::clock::EventClock;
use crate::closure::{analyze, ClosureOutcome, Plan};
use crate::scheduler::schedule_plans_with_alternatives;
use crate::survival::{
    apply_energy_interval, apply_plans, initial_inventory, model_with_inventory, BatteryLimits, FailureEvent, Resupply,
};

/// Explicit terminal state; never inferred from log text.
#[derive(Clone, Debug, PartialEq)]
pub enum EngineTermination {
    CompletedHorizon,
    Blocked(String),
    IncompleteAtHorizon(String),
}

/// One committed event.
#[derive(Clone, Debug, PartialEq)]
pub struct EngineTrace {
    pub at_s: f64,
    pub kind: String,
    pub inventory_before: BTreeMap<String, f64>,
    pub inventory_after: BTreeMap<String, f64>,
    pub plans: Vec<String>,
    pub resources: Vec<String>,
}

/// Full canonical result.
#[derive(Clone, Debug, PartialEq)]
pub struct EngineOutcome {
    pub termination: EngineTermination,
    pub survived_s: f64,
    pub trace: Vec<EngineTrace>,
    pub final_inventory: BTreeMap<String, f64>,
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

#[derive(Clone, Debug)]
enum TimelineItem {
    Failure(FailureEvent),
    Supply(Resupply),
}

struct RunningRepair {
    plans: Vec<Plan>,
    planning_base: BTreeMap<String, f64>,
    required: BTreeSet<String>,
    end_s: f64,
    label: String,
}

enum StartError {
    Blocked(String),
    Incomplete,
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
    let mut required = BTreeSet::new();
    for failure in pending {
        match analyze(&planning_model, &failure.asset, remaining_s) {
            ClosureOutcome::Restorable(plan) => {
                for step in &plan.steps {
                    if let Some(process) = current.processes.iter().find(|process| process.id == step.process) {
                        for asset in &process.occupies {
                            required.insert(asset.clone());
                        }
                    }
                    if let crate::closure::StepAction::Restore(asset) = &step.action {
                        required.insert(asset.clone());
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
    for asset in &required {
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
    let label = pending.iter().map(|failure| failure.asset.as_str()).collect::<Vec<_>>().join(",");
    Ok(RunningRepair { plans, planning_base, required, end_s: now_s + schedule.makespan_s, label })
}

fn plan_summary(plans: &[Plan]) -> Vec<String> {
    plans.iter().map(|plan| plan.steps.iter().map(|step| step.process.clone()).collect::<Vec<_>>().join("+")).collect()
}

/// Runs the canonical kernel.
pub fn run_engine(model: &Model, options: &EngineOptions<'_>) -> Result<EngineOutcome, EngineError> {
    let mut services: Vec<&Service> = Vec::new();
    for service_id in &options.services {
        let Some(service) = model.services.get(*service_id) else {
            return Err(EngineError(format!("unknown service {service_id}")));
        };
        services.push(service);
    }
    let mut current = model.clone();
    let mut inventory = initial_inventory(model);
    let mut clock = EventClock::new(options.horizon_s);
    let mut trace = Vec::new();

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
    if !pending.is_empty() {
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
        let next_event = timeline.get(index).map(|(at_s, _)| (*at_s).max(clock.now_s()));
        let next_completion = running.as_ref().map(|repair| repair.end_s);
        let next_s =
            [next_event, next_completion, Some(options.horizon_s)].into_iter().flatten().fold(f64::INFINITY, f64::min);
        if next_s > clock.horizon_s() + 1e-9 {
            break;
        }
        let gap = clock.advance_to(next_s);
        if let Err(offset) = apply_energy_interval(&current, &mut inventory, gap, options.battery, options.load_kw) {
            return Ok(EngineOutcome {
                termination: EngineTermination::Blocked("energy_depleted".into()),
                survived_s: clock.now_s() - gap + offset,
                trace,
                final_inventory: inventory,
            });
        }
        if let Some(repair) = running.as_ref() {
            if repair.end_s <= clock.now_s() + 1e-9 {
                let repair = running.take().expect("running repair");
                let before = inventory.clone();
                if let Err(reason) = apply_plans(&mut current, &mut inventory, &repair.plans, &repair.planning_base) {
                    return Ok(EngineOutcome {
                        termination: EngineTermination::Blocked(reason),
                        survived_s: clock.now_s(),
                        trace,
                        final_inventory: inventory,
                    });
                }
                trace.push(EngineTrace {
                    at_s: clock.now_s(),
                    kind: format!("repair_completed:{}", repair.label),
                    inventory_before: before,
                    inventory_after: inventory.clone(),
                    plans: plan_summary(&repair.plans),
                    resources: Vec::new(),
                });
                continue;
            }
        }
        if clock.now_s() >= options.horizon_s - 1e-9 {
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
        for (_, item) in &timeline[batch_start..index] {
            if let TimelineItem::Supply(supply) = item {
                let before = inventory.clone();
                *inventory.entry(supply.resource.clone()).or_insert(0.0) += supply.si;
                trace.push(EngineTrace {
                    at_s: clock.now_s(),
                    kind: format!("resupply:{}", supply.resource),
                    inventory_before: before,
                    inventory_after: inventory.clone(),
                    plans: Vec::new(),
                    resources: Vec::new(),
                });
            }
        }
        for (_, item) in &timeline[batch_start..index] {
            if let TimelineItem::Failure(failure) = item {
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
                trace.push(EngineTrace {
                    at_s: clock.now_s(),
                    kind: format!("failure:{}", failure.asset),
                    inventory_before: before.clone(),
                    inventory_after: inventory.clone(),
                    plans: Vec::new(),
                    resources: Vec::new(),
                });
                failed.push(failure.clone());
            }
        }
        if let Some(repair) = running.as_ref() {
            if let Some(hit) = failed.iter().find(|failure| repair.required.contains(&failure.asset)) {
                return Ok(EngineOutcome {
                    termination: EngineTermination::Blocked(format!(
                        "repair interrupted at {:.0}s: {} failed",
                        clock.now_s(),
                        hit.asset
                    )),
                    survived_s: clock.now_s(),
                    trace,
                    final_inventory: inventory,
                });
            }
        }
        pending.extend(failed);
    }

    let tail = clock.advance_to(options.horizon_s);
    if let Err(offset) = apply_energy_interval(&current, &mut inventory, tail, options.battery, options.load_kw) {
        return Ok(EngineOutcome {
            termination: EngineTermination::Blocked("energy_depleted".into()),
            survived_s: clock.now_s() - tail + offset,
            trace,
            final_inventory: inventory,
        });
    }
    trace.push(EngineTrace {
        at_s: clock.now_s(),
        kind: "horizon".into(),
        inventory_before: inventory.clone(),
        inventory_after: inventory.clone(),
        plans: Vec::new(),
        resources: Vec::new(),
    });
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
}
