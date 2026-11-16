// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Non-invasive canonical state observation at requested wall-clock times.
//!
//! Observation keeps the caller's horizon and inserts neutral zero-quantity
//! resupply probes at the requested times. Probes add no inventory and do not
//! change plans, resources or energy-settlement rules. Every observation also
//! returns the underlying unmodified `EngineOutcome` so callers can verify that
//! termination, survived time, final inventory and genuine event decisions are
//! unchanged. Active-repair identity is not yet exported by the engine and is
//! therefore either derived from the canonical trace or explicitly unknown.

use std::collections::BTreeMap;

use poie_ontology::model::Model;

use crate::engine::{run_engine, EngineError, EngineOptions, EngineOutcome, EngineTermination, EngineTrace};
use crate::survival::{initial_inventory, Resupply};

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
pub struct StateObservation {
    pub at_s: f64,
    pub status: ObservationStatus,
    pub energy_j: f64,
    pub inventory: BTreeMap<String, f64>,
    pub energy_generated_j: f64,
    pub energy_load_j: f64,
    pub energy_repair_j: f64,
    pub energy_curtailed_j: f64,
    /// `None` means the engine does not export enough state to decide.
    pub repair_active: Option<bool>,
    pub repair_detail: Option<String>,
    pub repair_state_source: &'static str,
    pub termination: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ObservationOutcome {
    pub outcome: EngineOutcome,
    pub observations: Vec<StateObservation>,
}

fn label(termination: &EngineTermination) -> String {
    match termination {
        EngineTermination::CompletedHorizon => "completed".into(),
        EngineTermination::Blocked(reason) => format!("blocked:{reason}"),
        EngineTermination::IncompleteAtHorizon(reason) => format!("incomplete:{reason}"),
    }
}

fn cumulative(trace: &[EngineTrace]) -> (f64, f64, f64, f64) {
    trace.iter().fold((0.0, 0.0, 0.0, 0.0), |acc, entry| {
        (
            acc.0 + entry.energy_generated_j,
            acc.1 + entry.energy_load_j,
            acc.2 + entry.energy_repair_j,
            acc.3 + entry.energy_curtailed_j,
        )
    })
}

fn active_repair(trace: &[EngineTrace], at_s: f64) -> (Option<bool>, Option<String>) {
    let completion =
        trace.iter().find(|entry| entry.kind.starts_with("repair_completed:") && entry.at_s >= at_s - 1e-6);
    let Some(completion) = completion else {
        // No future completion in this run: an interrupted or still-running
        // repair cannot be distinguished from no repair with the current API.
        return (None, None);
    };
    if (completion.at_s - at_s).abs() <= 1e-6 {
        return (Some(false), None);
    }
    let start = trace
        .iter()
        .rev()
        .find(|entry| entry.at_s < completion.at_s - 1e-9 && entry.kind.starts_with("failure:"))
        .map(|entry| entry.at_s)
        .unwrap_or(0.0);
    if at_s + 1e-9 >= start {
        let detail = completion.kind.split_once(':').map(|(_, value)| value.to_string());
        (Some(true), detail)
    } else {
        (Some(false), None)
    }
}

/// Observe canonical state at requested times without changing the requested
/// horizon. The returned outcome is the actual run that produced the samples.
pub fn observe(model: &Model, options: &EngineOptions<'_>, times_s: &[f64]) -> Result<ObservationOutcome, EngineError> {
    let mut run_options = options.clone();
    for &at_s in times_s {
        if at_s > 0.0 && at_s <= run_options.horizon_s + 1e-9 {
            run_options.resupplies.push(Resupply { at_s, resource: "electric-energy".into(), si: 0.0 });
        }
    }
    let outcome = run_engine(model, &run_options)?;
    let initial = initial_inventory(model);
    let (total_generated, total_load, total_repair, total_curtailed) = cumulative(&outcome.trace);
    let mut observations = Vec::with_capacity(times_s.len());
    for &at_s in times_s {
        let (status, inventory, generated, load, repair, curtailed) = if at_s <= 1e-9 {
            (ObservationStatus::Observed, initial.clone(), 0.0, 0.0, 0.0, 0.0)
        } else if at_s > outcome.survived_s + 1e-9 {
            (ObservationStatus::UnobservableAfterTermination, BTreeMap::new(), 0.0, 0.0, 0.0, 0.0)
        } else if let Some(entry) = outcome.trace.iter().rev().find(|entry| (entry.at_s - at_s).abs() <= 1e-9) {
            let upto = outcome.trace.iter().take_while(|item| item.at_s <= entry.at_s + 1e-9);
            let (g, l, r, c) = cumulative(&upto.cloned().collect::<Vec<_>>());
            (ObservationStatus::Observed, entry.inventory_after.clone(), g, l, r, c)
        } else {
            (ObservationStatus::UnobservableNotMaterialized, BTreeMap::new(), 0.0, 0.0, 0.0, 0.0)
        };
        let (repair_active, repair_detail) = active_repair(&outcome.trace, at_s);
        observations.push(StateObservation {
            at_s,
            status,
            energy_j: inventory.get("electric-energy").copied().unwrap_or(0.0),
            inventory,
            energy_generated_j: generated,
            energy_load_j: load,
            energy_repair_j: repair,
            energy_curtailed_j: curtailed,
            repair_active,
            repair_detail,
            repair_state_source: "DERIVED_FROM_CANONICAL_TRACE",
            termination: label(&outcome.termination),
        });
    }
    // The full-run totals are retained for reference; individual observed rows
    // use the cumulative trace prefix above.
    let _ = (total_generated, total_load, total_repair, total_curtailed);
    Ok(ObservationOutcome { outcome, observations })
}

pub fn observe_states(
    model: &Model,
    options: &EngineOptions<'_>,
    times_s: &[f64],
) -> Result<Vec<StateObservation>, EngineError> {
    Ok(observe(model, options, times_s)?.observations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::survival::BatteryLimits;
    use poie_ontology::model::Model;
    use std::path::{Path, PathBuf};

    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn options<'a>(horizon_s: f64) -> EngineOptions<'a> {
        EngineOptions {
            services: vec!["potable-water", "workshop-operability"],
            horizon_s,
            failures: Vec::new(),
            resupplies: Vec::new(),
            resources: vec!["robot-m1".into(), "lathe-1".into()],
            alternative_pool: vec!["robot-m1".into()],
            battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
            load_kw: 3.0,
        }
    }

    type SemanticEvent = (f64, String, BTreeMap<String, f64>, BTreeMap<String, f64>, Vec<String>, Vec<String>);

    fn semantic_trace(trace: &[EngineTrace]) -> Vec<SemanticEvent> {
        trace
            .iter()
            .filter(|entry| {
                !(entry.kind == "resupply:electric-energy"
                    && entry.energy_resupply_j == 0.0
                    && entry.inventory_before == entry.inventory_after)
            })
            .map(|entry| {
                (
                    entry.at_s,
                    entry.kind.clone(),
                    entry.inventory_before.clone(),
                    entry.inventory_after.clone(),
                    entry.plans.clone(),
                    entry.resources.clone(),
                )
            })
            .collect()
    }

    fn totals(trace: &[EngineTrace]) -> (f64, f64, f64, f64) {
        trace.iter().fold((0.0, 0.0, 0.0, 0.0), |acc, entry| {
            (
                acc.0 + entry.energy_generated_j,
                acc.1 + entry.energy_load_j,
                acc.2 + entry.energy_repair_j,
                acc.3 + entry.energy_curtailed_j,
            )
        })
    }

    fn assert_noninterference(model: &Model, options: &EngineOptions<'_>, samples: &[f64]) {
        let direct = run_engine(model, options).unwrap();
        let observed = observe(model, options, samples).unwrap();
        assert_eq!(direct.termination, observed.outcome.termination);
        assert_eq!(direct.survived_s, observed.outcome.survived_s);
        assert_eq!(direct.final_inventory, observed.outcome.final_inventory);
        assert_eq!(semantic_trace(&direct.trace), semantic_trace(&observed.outcome.trace));
        assert_eq!(totals(&direct.trace), totals(&observed.outcome.trace));
    }

    #[test]
    fn observer_samples_energy_and_repair_state() {
        let model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
        let observed =
            observe(&model, &options(200.0 * 3600.0), &[0.0, 3600.0, 7200.0, 10800.0, 284400.0]).unwrap().observations;
        let energy_kwh: Vec<f64> = observed.iter().map(|row| row.energy_j / 3_600_000.0).collect();
        assert_eq!(energy_kwh, vec![40.0, 39.5, 39.0, 38.0, 0.0]);
        assert_eq!(observed[0].repair_active, Some(true));
        assert_eq!(observed[1].repair_active, Some(true));
        assert_eq!(observed[2].repair_active, Some(true));
        assert_eq!(observed[1].repair_detail.as_deref(), Some("pump-p1"));
        assert_eq!(observed[3].repair_active, Some(false));
        assert_eq!(observed[4].repair_active, None);
        assert_eq!(observed[4].status, ObservationStatus::Observed);
        assert_eq!(observed[4].termination, "blocked:energy_depleted");
        assert_eq!(observed[4].energy_generated_j, 197.5 * 3_600_000.0);
        assert_eq!(observed[4].energy_load_j, 237.0 * 3_600_000.0);
        assert_eq!(observed[4].energy_repair_j, 0.5 * 3_600_000.0);
    }

    #[test]
    fn observation_preserves_original_horizon_and_termination() {
        let model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
        let observed = observe(&model, &options(200.0 * 3600.0), &[0.0, 284400.0]).unwrap();
        assert_eq!(observed.outcome.survived_s, 284400.0);
        assert!(matches!(observed.outcome.termination, EngineTermination::Blocked(_)));
        assert_eq!(observed.observations[1].termination, "blocked:energy_depleted");
    }

    #[test]
    fn unavailable_samples_are_explicitly_unobservable() {
        let model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
        let observed = observe(&model, &options(3600.0), &[0.0, 7200.0]).unwrap();
        assert_eq!(observed.observations[0].status, ObservationStatus::Observed);
        assert_eq!(observed.observations[1].status, ObservationStatus::UnobservableAfterTermination);
        assert_eq!(observed.observations[1].energy_j, 0.0);
        assert!(observed.observations[1].inventory.is_empty());
    }

    #[test]
    fn observation_noninterference_cases() {
        let model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
        // Energy exactly at horizon.
        assert_noninterference(&model, &options(79.0 * 3600.0), &[0.0, 3600.0, 79.0 * 3600.0]);
        // Early horizon and same-timestamp multiple failures.
        let mut early = options(3600.0);
        early.failures = vec![
            crate::survival::FailureEvent { at_s: 1800.0, asset: "pump-p1".into() },
            crate::survival::FailureEvent { at_s: 1800.0, asset: "filter-1".into() },
        ];
        assert_noninterference(&model, &early, &[0.0, 900.0, 1800.0]);
        // Repair interruption and completion exactly at horizon.
        let mut interrupted = options(90.0 * 86_400.0);
        interrupted.failures = vec![crate::survival::FailureEvent { at_s: 2.0 * 86_400.0, asset: "pump-p1".into() }];
        assert_noninterference(&model, &interrupted, &[0.0, 3600.0, 2.0 * 86_400.0]);
        let mut completion = options(3.0 * 3600.0);
        completion.failures = Vec::new();
        assert_noninterference(&model, &completion, &[0.0, 3600.0, 3.0 * 3600.0]);
    }
}
