// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Read-only canonical state observation at requested wall-clock times.
//!
//! The engine is not modified. Observation uses neutral zero-quantity resupply
//! probes at the requested times to force the canonical ledger to expose an
//! interval boundary. The probes add no inventory and do not change plans,
//! resources or energy-settlement rules; the final canonical state is verified
//! unchanged by the observatory runner. Active-repair state is derived from the
//! canonical trace because the current engine API does not export it directly.

use std::collections::BTreeMap;

use poie_ontology::model::Model;

use crate::engine::{run_engine, EngineError, EngineOptions, EngineTermination, EngineTrace};
use crate::survival::{initial_inventory, Resupply};

#[derive(Clone, Debug, PartialEq)]
pub struct StateObservation {
    pub at_s: f64,
    pub energy_j: f64,
    pub inventory: BTreeMap<String, f64>,
    pub energy_generated_j: f64,
    pub energy_load_j: f64,
    pub energy_repair_j: f64,
    pub energy_curtailed_j: f64,
    /// Derived from the canonical trace; not a direct engine field.
    pub repair_active: bool,
    pub repair_detail: Option<String>,
    pub repair_state_source: &'static str,
    pub termination: String,
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

fn active_repair(trace: &[EngineTrace], at_s: f64) -> (bool, Option<String>) {
    let completion = trace.iter().find(|entry| entry.kind.starts_with("repair_completed:") && entry.at_s > at_s + 1e-9);
    let Some(completion) = completion else {
        return (false, None);
    };
    let start = trace
        .iter()
        .rev()
        .find(|entry| entry.at_s < completion.at_s - 1e-9 && entry.kind.starts_with("failure:"))
        .map(|entry| entry.at_s)
        .unwrap_or(0.0);
    if at_s + 1e-9 >= start {
        let detail = completion.kind.split_once(':').map(|(_, value)| value.to_string());
        (true, detail)
    } else {
        (false, None)
    }
}

/// Observe canonical state at requested times. A single canonical run is made
/// with horizon equal to the last requested time and neutral zero supplies at
/// the sample times. No engine rule is changed.
pub fn observe_states(
    model: &Model,
    options: &EngineOptions<'_>,
    times_s: &[f64],
) -> Result<Vec<StateObservation>, EngineError> {
    let mut run_options = options.clone();
    let last = times_s.iter().copied().fold(0.0, f64::max);
    run_options.horizon_s = last.max(1.0);
    for &at_s in times_s {
        if at_s > 0.0 {
            run_options.resupplies.push(Resupply { at_s, resource: "electric-energy".into(), si: 0.0 });
        }
    }
    let outcome = run_engine(model, &run_options)?;
    let initial = initial_inventory(model);
    let (total_generated, total_load, total_repair, total_curtailed) = cumulative(&outcome.trace);
    let mut observations = Vec::with_capacity(times_s.len());
    for &at_s in times_s {
        let entry = if at_s <= 1e-9 {
            None
        } else {
            outcome.trace.iter().rev().find(|entry| (entry.at_s - at_s).abs() <= 1e-9)
        };
        let (inventory, generated, load, repair, curtailed) = if let Some(entry) = entry {
            let upto = outcome.trace.iter().take_while(|item| item.at_s <= entry.at_s + 1e-9);
            let (g, l, r, c) = cumulative(&upto.cloned().collect::<Vec<_>>());
            (entry.inventory_after.clone(), g, l, r, c)
        } else if at_s <= 1e-9 {
            (initial.clone(), 0.0, 0.0, 0.0, 0.0)
        } else {
            // No sample boundary was materialized; report the final state and
            // make the gap explicit by using the full-run totals.
            (outcome.final_inventory.clone(), total_generated, total_load, total_repair, total_curtailed)
        };
        let (repair_active, repair_detail) = active_repair(&outcome.trace, at_s);
        observations.push(StateObservation {
            at_s,
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
    Ok(observations)
}
