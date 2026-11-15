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
    apply_energy_interval_ledger, apply_plans, energy, initial_inventory, model_with_inventory, BatteryLimits,
    FailureEvent, Resupply,
};

/// Explicit terminal state; never inferred from log text.
#[derive(Clone, Debug, PartialEq)]
pub enum EngineTermination {
    CompletedHorizon,
    Blocked(String),
    IncompleteAtHorizon(String),
}

/// One capability choice and the assets it depends on during the repair.
#[derive(Clone, Debug, PartialEq)]
pub struct WitnessEntry {
    pub capability: String,
    pub provider: String,
    pub dependencies: Vec<String>,
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
    execution_dependencies: BTreeSet<String>,
    restoration_targets: BTreeSet<String>,
    resources: Vec<String>,
    witness: Vec<WitnessEntry>,
    end_s: f64,
    label: String,
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
    let label = pending.iter().map(|failure| failure.asset.as_str()).collect::<Vec<_>>().join(",");
    Ok(RunningRepair {
        plans,
        planning_base,
        execution_dependencies,
        restoration_targets,
        resources: schedule.steps.iter().flat_map(|step| step.resources.clone()).collect(),
        witness,
        end_s: now_s + schedule.makespan_s,
        label,
    })
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
        match apply_energy_interval_ledger(&current, &mut inventory, gap, options.battery, options.load_kw) {
            Ok(flow) => {
                interval_flow.generated_j += flow.generated_j;
                interval_flow.load_j += flow.load_j;
                interval_flow.curtailed_j += flow.curtailed_j;
            }
            Err(offset) => {
                return Ok(EngineOutcome {
                    termination: EngineTermination::Blocked("energy_depleted".into()),
                    survived_s: clock.now_s() - gap + offset,
                    trace,
                    final_inventory: inventory,
                });
            }
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
                    inventory_before: before,
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
                    inventory_before: before,
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
                failed.push(failure.clone());
            }
        }
        if let Some(repair) = running.as_ref() {
            if let Some(hit) = failed.iter().find(|failure| {
                repair.execution_dependencies.contains(&failure.asset)
                    || repair.restoration_targets.contains(&failure.asset)
            }) {
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
