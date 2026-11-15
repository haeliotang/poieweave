// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Canonical event execution kernel.
//!
//! One clock, one inventory ledger and one commit rule for initial repair,
//! failures, resupply and horizon. The kernel emits a typed termination, a
//! trace with before/after inventories, and the plan processes it applied.
//! Legacy `survival` entry points are not yet migrated; this is the target
//! implementation for S5-T.

use std::collections::BTreeMap;

use poie_ontology::model::{AssetStatus, Model, Service};

use crate::clock::EventClock;
use crate::closure::{analyze, ClosureOutcome, Plan};
use crate::scheduler::schedule_plans_with_alternatives;
use crate::survival::{
    apply_energy_interval, apply_plans, initial_inventory, model_with_inventory, restore_services, BatteryLimits,
    FailureEvent, Resupply,
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

enum TimelineItem<'a> {
    Failure(&'a FailureEvent),
    Supply(&'a Resupply),
}

fn plan_summary(plans: &[Plan]) -> Vec<String> {
    plans.iter().map(|plan| plan.steps.iter().map(|step| step.process.clone()).collect::<Vec<_>>().join("+")).collect()
}

fn commit_delta(
    inventory: &mut BTreeMap<String, f64>,
    planning_base: &BTreeMap<String, f64>,
    planned: &BTreeMap<String, f64>,
) -> Result<(), String> {
    let mut committed = inventory.clone();
    let mut keys: Vec<&String> = planning_base.keys().chain(planned.keys()).collect();
    keys.sort();
    keys.dedup();
    for resource in keys {
        let before = planning_base.get(resource).copied().unwrap_or(0.0);
        let after = planned.get(resource).copied().unwrap_or(0.0);
        *committed.entry(resource.clone()).or_insert(0.0) += after - before;
    }
    if let Some((resource, quantity)) = committed.iter().find(|(_, quantity)| **quantity < -1e-9) {
        return Err(format!("inventory negative for {resource} ({quantity:.3})"));
    }
    *inventory = committed;
    Ok(())
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
    let planning_base = inventory.clone();
    let mut pending_model = current.clone();
    let mut pending_inventory = planning_base.clone();
    let (repaired, repair_s) =
        match restore_services(&mut pending_model, &services, 0.0, options.horizon_s, &mut pending_inventory) {
            Ok(result) => result,
            Err(reason) => {
                return Ok(EngineOutcome {
                    termination: EngineTermination::Blocked(reason),
                    survived_s: 0.0,
                    trace,
                    final_inventory: inventory,
                });
            }
        };
    if repair_s > clock.remaining_s() + 1e-9 {
        return Ok(EngineOutcome {
            termination: EngineTermination::IncompleteAtHorizon("initial repair".into()),
            survived_s: options.horizon_s,
            trace,
            final_inventory: inventory,
        });
    }
    let elapsed = clock.advance_by(repair_s);
    if let Err(offset) = apply_energy_interval(&current, &mut inventory, elapsed, options.battery, options.load_kw) {
        return Ok(EngineOutcome {
            termination: EngineTermination::Blocked("energy_depleted".into()),
            survived_s: clock.now_s() - elapsed + offset,
            trace,
            final_inventory: inventory,
        });
    }
    let before = inventory.clone();
    if let Err(reason) = commit_delta(&mut inventory, &planning_base, &pending_inventory) {
        return Ok(EngineOutcome {
            termination: EngineTermination::Blocked(reason),
            survived_s: clock.now_s(),
            trace,
            final_inventory: inventory,
        });
    }
    current = pending_model;
    trace.push(EngineTrace {
        at_s: 0.0,
        kind: if repaired.is_empty() {
            "initial_repair:none".into()
        } else {
            format!("initial_repair:{}", repaired.join(","))
        },
        inventory_before: before,
        inventory_after: inventory.clone(),
        plans: Vec::new(),
        resources: Vec::new(),
    });

    let mut timeline: Vec<(f64, TimelineItem<'_>)> = Vec::new();
    for failure in &options.failures {
        timeline.push((failure.at_s, TimelineItem::Failure(failure)));
    }
    for supply in &options.resupplies {
        timeline.push((supply.at_s, TimelineItem::Supply(supply)));
    }
    timeline.sort_by(|left, right| {
        left.0.total_cmp(&right.0).then_with(|| {
            matches!(left.1, TimelineItem::Supply(_)).cmp(&matches!(right.1, TimelineItem::Supply(_))).reverse()
        })
    });

    let mut index = 0usize;
    while index < timeline.len() {
        let (at_s, _) = &timeline[index];
        if *at_s > clock.horizon_s() + 1e-9 {
            break;
        }
        let at = at_s.clamp(clock.now_s(), clock.horizon_s());
        let gap = clock.advance_to(at);
        if let Err(offset) = apply_energy_interval(&current, &mut inventory, gap, options.battery, options.load_kw) {
            return Ok(EngineOutcome {
                termination: EngineTermination::Blocked("energy_depleted".into()),
                survived_s: clock.now_s() - gap + offset,
                trace,
                final_inventory: inventory,
            });
        }
        let batch_start = index;
        while index < timeline.len() && timeline[index].0 == timeline[batch_start].0 {
            index += 1;
        }
        let items = &timeline[batch_start..index];
        if items.iter().any(|(_, item)| matches!(item, TimelineItem::Failure(_))) {
            let before = inventory.clone();
            let labels =
                items
                    .iter()
                    .filter_map(|(_, item)| {
                        if let TimelineItem::Failure(failure) = item {
                            Some(failure.asset.as_str())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(",");
            for (_, item) in items {
                let TimelineItem::Failure(failure) = item else { continue };
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
            }
            let planning_base = inventory.clone();
            let planning_model = model_with_inventory(&current, &planning_base);
            let mut plans = Vec::new();
            for (_, item) in items {
                let TimelineItem::Failure(failure) = item else { continue };
                match analyze(&planning_model, &failure.asset, clock.remaining_s()) {
                    ClosureOutcome::Restorable(plan) => plans.push(plan),
                    ClosureOutcome::Blocked(blocked) => {
                        return Ok(EngineOutcome {
                            termination: EngineTermination::Blocked(format!("{:?}", blocked.reasons)),
                            survived_s: clock.now_s(),
                            trace,
                            final_inventory: inventory,
                        });
                    }
                }
            }
            let mut pool = options.resources.clone();
            for plan in &plans {
                for step in &plan.steps {
                    if let Some(process) = current.processes.iter().find(|process| process.id == step.process) {
                        for asset in &process.occupies {
                            if !options.alternative_pool.contains(asset) && !pool.contains(asset) {
                                pool.push(asset.clone());
                            }
                        }
                    }
                }
            }
            let schedule = match schedule_plans_with_alternatives(
                &current,
                &plans,
                &pool,
                &options.alternative_pool,
                std::slice::from_ref(&options.alternative_pool),
            ) {
                Ok(schedule) => schedule,
                Err(error) => {
                    return Ok(EngineOutcome {
                        termination: EngineTermination::Blocked(format!("schedule failed: {error:?}")),
                        survived_s: clock.now_s(),
                        trace,
                        final_inventory: inventory,
                    });
                }
            };
            if schedule.makespan_s > clock.remaining_s() + 1e-9 {
                return Ok(EngineOutcome {
                    termination: EngineTermination::IncompleteAtHorizon(format!("repair {labels}")),
                    survived_s: options.horizon_s,
                    trace,
                    final_inventory: inventory,
                });
            }
            let repair_window = clock.advance_by(schedule.makespan_s);
            if let Err(offset) =
                apply_energy_interval(&current, &mut inventory, repair_window, options.battery, options.load_kw)
            {
                return Ok(EngineOutcome {
                    termination: EngineTermination::Blocked("energy_depleted".into()),
                    survived_s: clock.now_s() - repair_window + offset,
                    trace,
                    final_inventory: inventory,
                });
            }
            if let Err(reason) = apply_plans(&mut current, &mut inventory, &plans, &planning_base) {
                return Ok(EngineOutcome {
                    termination: EngineTermination::Blocked(reason),
                    survived_s: clock.now_s(),
                    trace,
                    final_inventory: inventory,
                });
            }
            trace.push(EngineTrace {
                at_s: clock.now_s(),
                kind: format!("failure:{labels}"),
                inventory_before: before,
                inventory_after: inventory.clone(),
                plans: plan_summary(&plans),
                resources: schedule.steps.iter().flat_map(|step| step.resources.clone()).collect(),
            });
        } else {
            let before = inventory.clone();
            for (_, item) in items {
                if let TimelineItem::Supply(supply) = item {
                    *inventory.entry(supply.resource.clone()).or_insert(0.0) += supply.si;
                }
            }
            trace.push(EngineTrace {
                at_s: clock.now_s(),
                kind: "resupply".into(),
                inventory_before: before,
                inventory_after: inventory.clone(),
                plans: Vec::new(),
                resources: Vec::new(),
            });
        }
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
    fn commit_delta_handles_keys_removed_from_the_plan() {
        let planning_base = BTreeMap::from([("sleeve-p1".to_string(), 1.0)]);
        let mut inventory = planning_base.clone();
        commit_delta(&mut inventory, &planning_base, &BTreeMap::new()).unwrap();
        assert_eq!(inventory.get("sleeve-p1").copied().unwrap_or(0.0), 0.0);
    }

    #[test]
    fn canonical_termination_is_typed() {
        let model = base08();
        let outcome =
            run_engine(&model, &options(vec![FailureEvent { at_s: 0.0, asset: "filter-1".into() }], 86_400.0)).unwrap();
        assert!(matches!(outcome.termination, EngineTermination::Blocked(_)));
    }
}
