// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Deterministic concurrent repair scheduler with Physical-Ontology resource
//! semantics.
//!
//! `occupies` means **all** listed assets are required, not alternatives. An
//! explicit [`ResourceRequirement`] can additionally name alternative groups;
//! inside a group exactly one resource is chosen and recorded. Plans remain
//! sequential internally.

use poie_ontology::model::Model;

use crate::closure::Plan;

/// One scheduled step with its interval and the resources it actually held.
#[derive(Clone, Debug, PartialEq)]
pub struct ScheduledStep {
    pub plan_index: usize,
    pub step_index: usize,
    pub process: String,
    pub start_s: f64,
    pub end_s: f64,
    pub resources: Vec<String>,
}

/// Result of scheduling a set of plans over a fixed resource pool.
#[derive(Clone, Debug, PartialEq)]
pub struct ScheduleReport {
    pub steps: Vec<ScheduledStep>,
    pub makespan_s: f64,
    pub plan_end_s: Vec<f64>,
    pub resource_available_s: Vec<f64>,
}

/// Resources that must all be free, plus groups where one is chosen.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResourceRequirement {
    pub required: Vec<String>,
    pub alternatives: Vec<Vec<String>>,
}

impl ResourceRequirement {
    pub fn required(resources: &[String]) -> ResourceRequirement {
        ResourceRequirement { required: resources.to_vec(), alternatives: Vec::new() }
    }

    pub fn one_of(resources: &[String]) -> ResourceRequirement {
        ResourceRequirement { required: Vec::new(), alternatives: vec![resources.to_vec()] }
    }
}

/// Scheduling failures are explicit; no fallback to a serial schedule.
#[derive(Clone, Debug, PartialEq)]
pub enum ScheduleError {
    NoResources,
    EmptyPlan(usize),
    UnknownProcess(String),
    MissingResource(String),
    EmptyAlternativeGroup,
}

/// A step ready for scheduling.
#[derive(Clone, Debug, PartialEq)]
pub struct SchedulableTask {
    pub plan_index: usize,
    pub step_index: usize,
    pub process: String,
    pub duration_s: f64,
    pub requirement: ResourceRequirement,
}

fn index_of(pool: &[String], resource: &str) -> Result<usize, ScheduleError> {
    pool.iter().position(|item| item == resource).ok_or_else(|| ScheduleError::MissingResource(resource.to_string()))
}

/// Schedules tasks over `resource_ids`. A task starts only when every required
/// resource and one resource from every alternative group are all free; all are
/// reserved until the task ends.
pub fn schedule_tasks(tasks: &[SchedulableTask], resource_ids: &[String]) -> Result<ScheduleReport, ScheduleError> {
    if resource_ids.is_empty() {
        return Err(ScheduleError::NoResources);
    }
    let mut available = vec![0.0_f64; resource_ids.len()];
    let mut plan_ready: Vec<f64> = Vec::new();
    let mut plan_next: Vec<usize> = Vec::new();
    for task in tasks {
        if task.plan_index >= plan_ready.len() {
            plan_ready.resize(task.plan_index + 1, 0.0);
            plan_next.resize(task.plan_index + 1, 0);
        }
        if task.step_index != plan_next[task.plan_index] {
            return Err(ScheduleError::EmptyPlan(task.plan_index));
        }
        plan_next[task.plan_index] += 1;
    }
    let mut plan_end: Vec<f64> = vec![0.0; plan_ready.len()];
    let mut next = vec![0usize; plan_ready.len()];
    let mut steps = Vec::with_capacity(tasks.len());
    for _ in 0..tasks.len() {
        let plan_index = (0..plan_ready.len())
            .filter(|index| next[*index] < plan_next[*index])
            .min_by(|left, right| plan_ready[*left].total_cmp(&plan_ready[*right]).then_with(|| left.cmp(right)))
            .expect("a plan has a remaining task");
        let task = tasks
            .iter()
            .find(|task| task.plan_index == plan_index && task.step_index == next[plan_index])
            .expect("task exists");
        let mut required = task
            .requirement
            .required
            .iter()
            .map(|resource| index_of(resource_ids, resource))
            .collect::<Result<Vec<_>, _>>()?;
        required.sort_unstable();
        required.dedup();
        let mut chosen = Vec::new();
        let mut start = plan_ready[plan_index];
        for index in &required {
            start = start.max(available[*index]);
            chosen.push(resource_ids[*index].clone());
        }
        for group in &task.requirement.alternatives {
            if group.is_empty() {
                return Err(ScheduleError::EmptyAlternativeGroup);
            }
            let index = group
                .iter()
                .map(|resource| index_of(resource_ids, resource))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .min_by(|left, right| available[*left].total_cmp(&available[*right]).then_with(|| left.cmp(right)))
                .expect("group is non-empty");
            start = start.max(available[index]);
            chosen.push(resource_ids[index].clone());
        }
        let end = start + task.duration_s;
        for resource in &chosen {
            available[index_of(resource_ids, resource)?] = end;
        }
        plan_end[plan_index] = end;
        plan_ready[plan_index] = end;
        next[plan_index] += 1;
        steps.push(ScheduledStep {
            plan_index,
            step_index: task.step_index,
            process: task.process.clone(),
            start_s: start,
            end_s: end,
            resources: chosen,
        });
    }
    let makespan_s = steps.iter().map(|step| step.end_s).fold(0.0_f64, f64::max);
    Ok(ScheduleReport { steps, makespan_s, plan_end_s: plan_end, resource_available_s: available })
}

/// Strict Physical-Ontology mode: every `process.occupies` entry is required.
pub fn schedule_plans(model: &Model, plans: &[Plan], resource_ids: &[String]) -> Result<ScheduleReport, ScheduleError> {
    let mut tasks = Vec::new();
    for (plan_index, plan) in plans.iter().enumerate() {
        if plan.steps.is_empty() {
            return Err(ScheduleError::EmptyPlan(plan_index));
        }
        for (step_index, step) in plan.steps.iter().enumerate() {
            let process = model
                .processes
                .iter()
                .find(|process| process.id == step.process)
                .ok_or_else(|| ScheduleError::UnknownProcess(step.process.clone()))?;
            tasks.push(SchedulableTask {
                plan_index,
                step_index,
                process: step.process.clone(),
                duration_s: step.duration_s,
                requirement: ResourceRequirement::required(&process.occupies),
            });
        }
    }
    schedule_tasks(&tasks, resource_ids)
}

/// Explicit-policy mode: the caller declares which resources are required and
/// which form an alternative group (for example a robot pool). The chosen
/// resource is written into [`ScheduleReport::steps`].
pub fn schedule_plans_with_requirement(
    model: &Model,
    plans: &[Plan],
    resource_ids: &[String],
    requirement: &ResourceRequirement,
) -> Result<ScheduleReport, ScheduleError> {
    let mut tasks = Vec::new();
    for (plan_index, plan) in plans.iter().enumerate() {
        if plan.steps.is_empty() {
            return Err(ScheduleError::EmptyPlan(plan_index));
        }
        for (step_index, step) in plan.steps.iter().enumerate() {
            model
                .processes
                .iter()
                .find(|process| process.id == step.process)
                .ok_or_else(|| ScheduleError::UnknownProcess(step.process.clone()))?;
            tasks.push(SchedulableTask {
                plan_index,
                step_index,
                process: step.process.clone(),
                duration_s: step.duration_s,
                requirement: requirement.clone(),
            });
        }
    }
    schedule_tasks(&tasks, resource_ids)
}

/// Explicit alternative policy without erasing other required resources.
///
/// `process.occupies` entries listed in `replaceable` are replaced by the
/// alternative `groups`; every other `occupies` entry stays required. This is
/// the safe way to model a robot pool: tools and fixtures remain AND
/// requirements.
pub fn schedule_plans_with_alternatives(
    model: &Model,
    plans: &[Plan],
    resource_ids: &[String],
    replaceable: &[String],
    groups: &[Vec<String>],
) -> Result<ScheduleReport, ScheduleError> {
    let mut tasks = Vec::new();
    for (plan_index, plan) in plans.iter().enumerate() {
        if plan.steps.is_empty() {
            return Err(ScheduleError::EmptyPlan(plan_index));
        }
        for (step_index, step) in plan.steps.iter().enumerate() {
            let process = model
                .processes
                .iter()
                .find(|process| process.id == step.process)
                .ok_or_else(|| ScheduleError::UnknownProcess(step.process.clone()))?;
            let required: Vec<String> =
                process.occupies.iter().filter(|resource| !replaceable.contains(resource)).cloned().collect();
            tasks.push(SchedulableTask {
                plan_index,
                step_index,
                process: step.process.clone(),
                duration_s: step.duration_s,
                requirement: ResourceRequirement { required, alternatives: groups.to_vec() },
            });
        }
    }
    schedule_tasks(&tasks, resource_ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use crate::closure::{Plan, Step, StepAction};

    fn model() -> Model {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
        for process in &mut model.processes {
            if process.id == "replace-sleeve-p1" {
                process.occupies = vec!["robot-m1".into(), "tool-t1".into()];
            }
            if process.id == "replace-lathe-nut" {
                process.occupies = vec!["robot-m2".into()];
            }
        }
        model
    }

    fn robots(count: usize) -> Vec<String> {
        (1..=count).map(|index| format!("robot-m{index}")).collect()
    }

    fn plan(duration_s: f64, process: &str) -> Plan {
        Plan {
            steps: vec![Step { process: process.into(), action: StepAction::Restore("pump-p1".into()), duration_s }],
            total_duration_s: duration_s,
            remaining_inventory: BTreeMap::new(),
        }
    }

    #[test]
    fn strict_mode_requires_every_occupied_resource() {
        let model = model();
        let plans = vec![plan(3.0 * 3600.0, "replace-sleeve-p1")];
        let only_robot = schedule_plans(&model, &plans, &["robot-m1".into()]);
        assert!(matches!(only_robot, Err(ScheduleError::MissingResource(_))));
        let both = schedule_plans(&model, &plans, &["robot-m1".into(), "tool-t1".into()]).unwrap();
        assert_eq!(both.makespan_s, 3.0 * 3600.0);
        assert_eq!(both.steps[0].resources, vec!["robot-m1", "tool-t1"]);
    }

    #[test]
    fn alternative_robot_pool_overlaps_plans() {
        let model = model();
        let plans = vec![plan(3.0 * 3600.0, "replace-sleeve-p1"), plan(2.0 * 3600.0, "replace-lathe-nut")];
        let requirement = ResourceRequirement::one_of(&robots(2));
        let serial = schedule_plans_with_requirement(&model, &plans, &robots(2), &requirement).unwrap();
        assert_eq!(serial.makespan_s, 3.0 * 3600.0);
        assert_ne!(serial.steps[0].resources, serial.steps[1].resources);
    }

    #[test]
    fn alternatives_do_not_erase_required_tools() {
        let model = model();
        let plans = vec![plan(3.0 * 3600.0, "replace-sleeve-p1")];
        let pool = robots(2);
        let result = schedule_plans_with_alternatives(&model, &plans, &pool, &pool, std::slice::from_ref(&pool));
        assert!(matches!(result, Err(ScheduleError::MissingResource(ref resource)) if resource == "tool-t1"));
    }

    #[test]
    fn chosen_resources_never_overlap() {
        let model = model();
        let plans = vec![
            plan(3.0 * 3600.0, "replace-sleeve-p1"),
            plan(2.0 * 3600.0, "replace-lathe-nut"),
            plan(2.0 * 3600.0, "replace-sleeve-p1"),
        ];
        let requirement = ResourceRequirement::one_of(&robots(2));
        let report = schedule_plans_with_requirement(&model, &plans, &robots(2), &requirement).unwrap();
        for resource in robots(2) {
            let mut intervals: Vec<(f64, f64)> = report
                .steps
                .iter()
                .filter(|step| step.resources.contains(&resource))
                .map(|step| (step.start_s, step.end_s))
                .collect();
            intervals.sort_by(|left, right| left.0.total_cmp(&right.0));
            for pair in intervals.windows(2) {
                assert!(pair[1].0 >= pair[0].1, "{resource} overlaps: {:?}", pair);
            }
        }
    }

    #[test]
    fn a_plan_remains_sequential_across_its_steps() {
        let model = model();
        let mut multi = plan(3600.0, "replace-lathe-nut");
        multi.steps.push(Step {
            process: "replace-lathe-nut".into(),
            action: StepAction::Restore("pump-p1".into()),
            duration_s: 3600.0,
        });
        multi.total_duration_s = 7200.0;
        let requirement = ResourceRequirement::one_of(&robots(2));
        let report = schedule_plans_with_requirement(&model, &[multi], &robots(2), &requirement).unwrap();
        assert_eq!(report.makespan_s, 7200.0);
        assert_eq!(report.steps[1].start_s, 3600.0);
    }
}
