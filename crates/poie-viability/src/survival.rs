// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 first slice: failure-envelope recovery coverage.
//!
//! Given a service, a horizon and a maximum number of simultaneously failed
//! assets, this module enumerates failure combinations and asks whether every
//! service requirement can be restored by [`crate::closure::analyze`].
//!
//! The analysis is intentionally labelled *independent*: each required asset is
//! checked against the same initial inventory, so two repairs that compete for
//! the same lot can both appear recoverable. This is a coverage envelope, not a
//! joint repair schedule. The report records that limitation.

use std::collections::BTreeMap;

use poie_ontology::model::{AssetStatus, Lot, LotQuality, Model, Service};

use crate::closure::{analyze, BlockReason, ClosureOutcome, StepAction};

#[derive(Clone, Debug, PartialEq)]
pub struct FailureCase {
    pub failed: Vec<String>,
    pub recoverable: bool,
    pub max_plan_s: f64,
    pub blocked_by: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FailureEnvelope {
    pub service: String,
    pub horizon_s: f64,
    pub cases: Vec<FailureCase>,
    pub recoverable: usize,
}

impl FailureEnvelope {
    pub fn total(&self) -> usize {
        self.cases.len()
    }

    pub fn coverage(&self) -> f64 {
        if self.cases.is_empty() {
            0.0
        } else {
            self.recoverable as f64 / self.cases.len() as f64
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnvelopeError(pub String);

impl std::fmt::Display for EnvelopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for EnvelopeError {}

/// Enumerate failure subsets of the model's assets, up to `max_failures`, and
/// report per-combination recovery coverage for one service.
pub fn failure_envelope(
    model: &Model,
    service: &str,
    horizon_s: f64,
    max_failures: usize,
) -> Result<FailureEnvelope, EnvelopeError> {
    let Some(service) = model.services.get(service) else {
        return Err(EnvelopeError(format!("unknown service {service}")));
    };
    let assets: Vec<String> = model.assets.keys().cloned().collect();
    let mut cases = Vec::new();
    for size in 0..=max_failures.min(assets.len()) {
        for combination in combinations(&assets, size) {
            cases.push(evaluate_case(model, service, &combination, horizon_s));
        }
    }
    let recoverable = cases.iter().filter(|case| case.recoverable).count();
    Ok(FailureEnvelope { service: service.id.clone(), horizon_s, cases, recoverable })
}

fn evaluate_case(
    model: &Model,
    service: &poie_ontology::model::Service,
    failed: &[String],
    horizon_s: f64,
) -> FailureCase {
    let mut failed_model = model.clone();
    for asset in failed {
        if let Some(asset) = failed_model.assets.get_mut(asset) {
            asset.status = AssetStatus::Failed;
        }
    }
    let mut max_plan_s: f64 = 0.0;
    let mut blocked_by = None;

    for asset in &service.requires {
        match analyze(&failed_model, asset, horizon_s) {
            ClosureOutcome::Restorable(plan) => max_plan_s = max_plan_s.max(plan.total_duration_s),
            ClosureOutcome::Blocked(blocked) => {
                blocked_by = Some(blocked.reasons.first().map(reason_text).unwrap_or_else(|| "blocked".into()));
                break;
            }
        }
    }
    if blocked_by.is_none() && !service.requires_any.is_empty() {
        let mut alternatives = Vec::new();
        for asset in &service.requires_any {
            match analyze(&failed_model, asset, horizon_s) {
                ClosureOutcome::Restorable(plan) => alternatives.push(plan.total_duration_s),
                ClosureOutcome::Blocked(blocked) => {
                    if let Some(reason) = blocked.reasons.first() {
                        alternatives.push(-1.0);
                        if blocked_by.is_none() {
                            blocked_by = Some(reason_text(reason));
                        }
                    }
                }
            }
        }
        if alternatives.iter().all(|duration| *duration < 0.0) {
            blocked_by.get_or_insert_with(|| "no alternative service asset".into());
        } else {
            blocked_by = None;
            max_plan_s =
                max_plan_s.max(alternatives.into_iter().filter(|duration| *duration >= 0.0).fold(0.0, f64::max));
        }
    }

    FailureCase { failed: failed.to_vec(), recoverable: blocked_by.is_none(), max_plan_s, blocked_by }
}

fn reason_text(reason: &BlockReason) -> String {
    reason.to_string()
}

fn combinations(items: &[String], size: usize) -> Vec<Vec<String>> {
    if size == 0 {
        return vec![Vec::new()];
    }
    if size > items.len() {
        return Vec::new();
    }
    let mut result = Vec::new();
    for (index, item) in items.iter().enumerate() {
        for mut rest in combinations(&items[index + 1..], size - 1) {
            rest.insert(0, item.clone());
            result.push(rest);
        }
    }
    result
}

#[derive(Clone, Debug, PartialEq)]
pub struct FailureEvent {
    pub at_s: f64,
    pub asset: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SurvivalLogEntry {
    pub at_s: f64,
    pub failed: Option<String>,
    pub repaired: Vec<String>,
    pub energy_after: f64,
    pub blocked_by: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SurvivalReport {
    pub service: String,
    pub horizon_s: f64,
    pub survived_s: f64,
    pub log: Vec<SurvivalLogEntry>,
    pub final_inventory: BTreeMap<String, f64>,
}

/// A greedy, single-threaded recovery policy over scheduled failures.
///
/// At t=0 the service is repaired if needed. Failures are applied in time order;
/// before each event, PV energy is added from Base-05's modelled recharge
/// process if `pv-1` is available. Every service requirement is then repaired
/// with S1 closure against the current inventory, and the plan's remaining
/// inventory becomes the next state. The first blocked event ends the run.
pub fn simulate_longevity(
    model: &Model,
    service_id: &str,
    horizon_s: f64,
    events: &[FailureEvent],
) -> Result<SurvivalReport, EnvelopeError> {
    let Some(service) = model.services.get(service_id) else {
        return Err(EnvelopeError(format!("unknown service {service_id}")));
    };
    let mut current = model.clone();
    let mut inventory = initial_inventory(model);
    let mut log = Vec::new();
    let mut now = 0.0;

    match restore_service(&mut current, service, 0.0, horizon_s, &mut inventory) {
        Ok(repaired) => log.push(SurvivalLogEntry {
            at_s: 0.0,
            failed: None,
            repaired,
            energy_after: energy(&inventory),
            blocked_by: None,
        }),
        Err(reason) => {
            log.push(SurvivalLogEntry {
                at_s: 0.0,
                failed: None,
                repaired: Vec::new(),
                energy_after: energy(&inventory),
                blocked_by: Some(reason),
            });
            return Ok(SurvivalReport {
                service: service.id.clone(),
                horizon_s,
                survived_s: 0.0,
                log,
                final_inventory: inventory,
            });
        }
    }

    let mut ordered = events.to_vec();
    ordered.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    for event in ordered {
        let at = event.at_s.clamp(now, horizon_s);
        add_recharge(&current, &mut inventory, at - now);
        now = at;
        if let Some(asset) = current.assets.get_mut(&event.asset) {
            asset.status = AssetStatus::Failed;
        } else {
            return Err(EnvelopeError(format!("unknown failed asset {}", event.asset)));
        }
        match restore_service(&mut current, service, now, horizon_s, &mut inventory) {
            Ok(repaired) => log.push(SurvivalLogEntry {
                at_s: now,
                failed: Some(event.asset),
                repaired,
                energy_after: energy(&inventory),
                blocked_by: None,
            }),
            Err(reason) => {
                log.push(SurvivalLogEntry {
                    at_s: now,
                    failed: Some(event.asset),
                    repaired: Vec::new(),
                    energy_after: energy(&inventory),
                    blocked_by: Some(reason),
                });
                return Ok(SurvivalReport {
                    service: service.id.clone(),
                    horizon_s,
                    survived_s: now,
                    log,
                    final_inventory: inventory,
                });
            }
        }
    }
    add_recharge(&current, &mut inventory, horizon_s - now);
    Ok(SurvivalReport {
        service: service.id.clone(),
        horizon_s,
        survived_s: horizon_s,
        log,
        final_inventory: inventory,
    })
}

fn restore_service(
    model: &mut Model,
    service: &Service,
    now_s: f64,
    horizon_s: f64,
    inventory: &mut BTreeMap<String, f64>,
) -> Result<Vec<String>, String> {
    let remaining = (horizon_s - now_s).max(0.0);
    let mut repaired = Vec::new();
    for asset in &service.requires {
        if model.assets.get(asset).is_some_and(|item| item.status == AssetStatus::Available) {
            continue;
        }
        repair_one(model, asset, remaining, inventory, &mut repaired)?;
    }
    if !service.requires_any.is_empty() {
        let already_available = service
            .requires_any
            .iter()
            .any(|asset| model.assets.get(asset).is_some_and(|item| item.status == AssetStatus::Available));
        if !already_available {
            let mut best: Option<String> = None;
            for asset in &service.requires_any {
                let snapshot = (model.clone(), inventory.clone());
                match repair_one(model, asset, remaining, inventory, &mut Vec::new()) {
                    Ok(()) => {
                        best = Some(asset.clone());
                        break;
                    }
                    Err(reason) => {
                        *model = snapshot.0;
                        *inventory = snapshot.1;
                        if best.is_none() {
                            best = Some(format!("blocked:{reason}"));
                        }
                    }
                }
            }
            match best {
                Some(asset) if !asset.starts_with("blocked:") => repaired.push(asset),
                _ => return Err(best.unwrap_or_else(|| "no alternative service asset".into())),
            }
        }
    }
    Ok(repaired)
}

fn repair_one(
    model: &mut Model,
    asset: &str,
    remaining_s: f64,
    inventory: &mut BTreeMap<String, f64>,
    repaired: &mut Vec<String>,
) -> Result<(), String> {
    let with_inventory = model_with_inventory(model, inventory);
    match analyze(&with_inventory, asset, remaining_s) {
        ClosureOutcome::Restorable(plan) => {
            for step in &plan.steps {
                if let StepAction::Restore(restored) = &step.action {
                    if let Some(asset) = model.assets.get_mut(restored) {
                        asset.status = AssetStatus::Available;
                    }
                }
            }
            *inventory = plan.remaining_inventory;
            repaired.push(asset.to_string());
            Ok(())
        }
        ClosureOutcome::Blocked(blocked) => {
            Err(blocked.reasons.first().map(ToString::to_string).unwrap_or_else(|| "blocked".into()))
        }
    }
}

fn model_with_inventory(model: &Model, inventory: &BTreeMap<String, f64>) -> Model {
    let mut clone = model.clone();
    let mut lots = Vec::new();
    for (index, (resource, quantity)) in inventory.iter().enumerate() {
        if *quantity > 1e-9 {
            lots.push(Lot {
                id: format!("survival-{index}-{resource}"),
                resource: resource.clone(),
                si: *quantity,
                quality: LotQuality::Accepted,
            });
        }
    }
    clone.lots = lots;
    clone
}

fn initial_inventory(model: &Model) -> BTreeMap<String, f64> {
    let mut inventory = BTreeMap::new();
    for lot in &model.lots {
        if lot.quality == LotQuality::Accepted && lot.si > 0.0 {
            *inventory.entry(lot.resource.clone()).or_insert(0.0) += lot.si;
        }
    }
    inventory
}

fn energy(inventory: &BTreeMap<String, f64>) -> f64 {
    inventory.get("electric-energy").copied().unwrap_or(0.0)
}

fn recharge_kwh_per_s(model: &Model) -> Option<f64> {
    let process = model.processes.iter().find(|process| process.id == "recharge-energy")?;
    let duration = process.duration_s?;
    let output = process.outputs.iter().find(|output| output.resource == "electric-energy")?;
    if duration <= 0.0 {
        return None;
    }
    Some(output.si / duration)
}

fn add_recharge(model: &Model, inventory: &mut BTreeMap<String, f64>, elapsed_s: f64) {
    if elapsed_s <= 0.0 {
        return;
    }
    if !model.assets.get("pv-1").is_some_and(|asset| asset.status == AssetStatus::Available) {
        return;
    }
    if let Some(rate) = recharge_kwh_per_s(model) {
        *inventory.entry("electric-energy".into()).or_insert(0.0) += rate * elapsed_s;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn base05() -> Model {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        Model::load(&root.join("scenarios/base-05/model.toml")).expect("Base-05")
    }

    #[test]
    fn base05_empty_failure_is_recoverable() {
        let envelope = failure_envelope(&base05(), "potable-water", 12.0 * 3600.0, 0).unwrap();
        assert_eq!(envelope.total(), 1);
        assert!(envelope.cases[0].recoverable);
    }

    #[test]
    fn filter_failure_is_blocked() {
        let envelope = failure_envelope(&base05(), "potable-water", 12.0 * 3600.0, 1).unwrap();
        assert!(envelope.cases.iter().any(|case| case.failed == vec!["filter-1".to_string()] && !case.recoverable));
    }

    #[test]
    fn bus_failure_is_blocked() {
        let envelope = failure_envelope(&base05(), "potable-water", 12.0 * 3600.0, 1).unwrap();
        assert!(envelope.cases.iter().any(|case| case.failed == vec!["bus-1".to_string()] && !case.recoverable));
    }

    #[test]
    fn coverage_is_below_one() {
        let envelope = failure_envelope(&base05(), "potable-water", 12.0 * 3600.0, 1).unwrap();
        assert!(envelope.coverage() < 1.0);
        assert!(envelope.recoverable > 0);
    }

    #[test]
    fn initial_repair_survives_without_failures() {
        let report = simulate_longevity(&base05(), "potable-water", 30.0 * 86400.0, &[]).unwrap();
        assert_eq!(report.survived_s, 30.0 * 86400.0);
        assert_eq!(report.log[0].at_s, 0.0);
        assert!(report.log[0].blocked_by.is_none());
    }

    #[test]
    fn filter_failure_stops_the_long_horizon() {
        let report = simulate_longevity(
            &base05(),
            "potable-water",
            30.0 * 86400.0,
            &[FailureEvent { at_s: 10.0 * 86400.0, asset: "filter-1".into() }],
        )
        .unwrap();
        assert_eq!(report.survived_s, 10.0 * 86400.0);
        assert!(report.log.last().unwrap().blocked_by.is_some());
    }

    #[test]
    fn repeated_pump_failures_eventually_exhaust_stock() {
        let events: Vec<FailureEvent> = (0..9)
            .map(|index| FailureEvent { at_s: f64::from(index) * 10.0 * 86400.0, asset: "pump-p1".into() })
            .collect();
        let report = simulate_longevity(&base05(), "potable-water", 90.0 * 86400.0, &events).unwrap();
        assert!(report.survived_s < 90.0 * 86400.0);
        assert!(report.log.iter().any(|entry| entry.blocked_by.is_some()));
    }
}
