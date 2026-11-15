// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Deterministic concurrent repair scheduler.
//!
//! Plans remain sequential internally; steps of different plans may run at the
//! same time when they need different resources. This is intentionally a small
//! list scheduler, not a preemptive or optimal scheduler.

use poie_ontology::model::Model;

use crate::closure::Plan;

/// One scheduled step with its wall-clock interval and chosen resources.
#[derive(Clone, Debug, PartialEq)]
pub struct ScheduledStep {
    pub plan_index: usize,
    pub step_index: usize,
    pub process: String,
    pub start_s: f64,
    pub end_s: f64,
    pub resources: Vec<String>,
}

/// Result of scheduling a set of plans over a fixed resource set.
#[derive(Clone, Debug, PartialEq)]
pub struct ScheduleReport {
    pub steps: Vec<ScheduledStep>,
    pub makespan_s: f64,
    pub plan_end_s: Vec<f64>,
    pub resource_available_s: Vec<f64>,
}

/// Scheduling failures are explicit; no fallback to a serial schedule.
#[derive(Clone, Debug, PartialEq)]
pub enum ScheduleError {
    NoResources,
    EmptyPlan(usize),
    UnknownProcess(String),
}

/// Schedules every plan step over `resource_ids` (for example `robot-m1`).
///
/// A step that occupies one or more of those resources starts at the earliest
/// time at least one is free. A step with no matching resource is modeled as
/// not consuming a shared resource and may run immediately after its plan's
/// previous step.
pub fn schedule_plans(model: &Model, plans: &[Plan], resource_ids: &[String]) -> Result<ScheduleReport, ScheduleError> {
    if resource_ids.is_empty() {
        return Err(ScheduleError::NoResources);
    }
    let mut available = vec![0.0_f64; resource_ids.len()];
    let mut plan_next = vec![0usize; plans.len()];
    let mut plan_ready = vec![0.0_f64; plans.len()];
    let mut plan_end = vec![0.0_f64; plans.len()];
    let mut steps = Vec::new();
    let total_steps: usize = plans.iter().map(|plan| plan.steps.len()).sum();
    if plans.iter().any(|plan| plan.steps.is_empty()) {
        let index = plans.iter().position(|plan| plan.steps.is_empty()).unwrap_or(0);
        return Err(ScheduleError::EmptyPlan(index));
    }

    for _ in 0..total_steps {
        let plan_index = (0..plans.len())
            .filter(|index| plan_next[*index] < plans[*index].steps.len())
            .min_by(|left, right| plan_ready[*left].total_cmp(&plan_ready[*right]).then_with(|| left.cmp(right)))
            .expect("a plan has a remaining step");
        let step = &plans[plan_index].steps[plan_next[plan_index]];
        let process = model
            .processes
            .iter()
            .find(|process| process.id == step.process)
            .ok_or_else(|| ScheduleError::UnknownProcess(step.process.clone()))?;
        let mut occupied: Vec<usize> = process
            .occupies
            .iter()
            .filter_map(|asset| resource_ids.iter().position(|resource| resource == asset))
            .collect();
        occupied.sort_unstable();
        occupied.dedup();
        let (start_s, end_s, chosen) = if occupied.is_empty() {
            (plan_ready[plan_index], plan_ready[plan_index] + step.duration_s, Vec::new())
        } else {
            let resource_index = *occupied
                .iter()
                .min_by(|left, right| available[**left].total_cmp(&available[**right]).then_with(|| left.cmp(right)))
                .expect("occupied is non-empty");
            let start_s = plan_ready[plan_index].max(available[resource_index]);
            let end_s = start_s + step.duration_s;
            available[resource_index] = end_s;
            (start_s, end_s, vec![resource_ids[resource_index].clone()])
        };
        plan_end[plan_index] = end_s;
        plan_ready[plan_index] = end_s;
        plan_next[plan_index] += 1;
        steps.push(ScheduledStep {
            plan_index,
            step_index: plan_next[plan_index] - 1,
            process: step.process.clone(),
            start_s,
            end_s,
            resources: chosen,
        });
    }

    let makespan_s = steps.iter().map(|step| step.end_s).fold(0.0_f64, f64::max);
    Ok(ScheduleReport { steps, makespan_s, plan_end_s: plan_end, resource_available_s: available })
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
            if process.id == "replace-sleeve-p1" || process.id == "replace-lathe-nut" {
                process.occupies = vec!["robot-m1".into(), "robot-m2".into(), "robot-m3".into()];
            }
        }
        model
    }

    fn plan(duration_s: f64, process: &str) -> Plan {
        Plan {
            steps: vec![Step { process: process.into(), action: StepAction::Restore("pump-p1".into()), duration_s }],
            total_duration_s: duration_s,
            remaining_inventory: BTreeMap::new(),
        }
    }

    fn robots(count: usize) -> Vec<String> {
        (1..=count).map(|index| format!("robot-m{index}")).collect()
    }

    #[test]
    fn two_plans_run_in_parallel_on_two_robots() {
        let model = model();
        let plans = vec![plan(3.0 * 3600.0, "replace-sleeve-p1"), plan(2.0 * 3600.0, "replace-lathe-nut")];
        let serial = schedule_plans(&model, &plans, &robots(1)).unwrap();
        assert_eq!(serial.makespan_s, 5.0 * 3600.0);
        let parallel = schedule_plans(&model, &plans, &robots(2)).unwrap();
        assert_eq!(parallel.makespan_s, 3.0 * 3600.0);
    }

    #[test]
    fn three_plans_need_two_robots_for_a_four_hour_makespan() {
        let model = model();
        let plans = vec![
            plan(3.0 * 3600.0, "replace-sleeve-p1"),
            plan(2.0 * 3600.0, "replace-lathe-nut"),
            plan(2.0 * 3600.0, "replace-sleeve-p1"),
        ];
        assert_eq!(schedule_plans(&model, &plans, &robots(1)).unwrap().makespan_s, 7.0 * 3600.0);
        assert_eq!(schedule_plans(&model, &plans, &robots(2)).unwrap().makespan_s, 4.0 * 3600.0);
        assert_eq!(schedule_plans(&model, &plans, &robots(3)).unwrap().makespan_s, 3.0 * 3600.0);
    }

    #[test]
    fn a_plan_remains_sequential_across_its_steps() {
        let model = model();
        let mut multi = plan(3600.0, "replace-sleeve-p1");
        multi.steps.push(Step {
            process: "replace-sleeve-p1".into(),
            action: StepAction::Restore("pump-p1".into()),
            duration_s: 3600.0,
        });
        multi.total_duration_s = 7200.0;
        let report = schedule_plans(&model, &[multi], &robots(2)).unwrap();
        assert_eq!(report.makespan_s, 7200.0);
        assert_eq!(report.steps[0].end_s, 3600.0);
        assert_eq!(report.steps[1].start_s, 3600.0);
    }
}
