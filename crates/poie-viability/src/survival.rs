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

use crate::closure::{analyze, BlockReason, ClosureOutcome, Plan, StepAction};
use poie_digest::sha256_of;
use serde::Serialize;

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BatteryLimits {
    pub capacity_kwh: f64,
    pub charge_kw: f64,
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
    simulate_longevity_with_limits(model, service_id, horizon_s, events, None)
}

/// Same policy, with an optional battery capacity and charge-power cap.
/// `BatteryLimits` is a study parameter because the ontology model has no
/// battery-capacity field yet.
pub fn simulate_longevity_with_limits(
    model: &Model,
    service_id: &str,
    horizon_s: f64,
    events: &[FailureEvent],
    battery: Option<BatteryLimits>,
) -> Result<SurvivalReport, EnvelopeError> {
    simulate_multi_service_longevity_with_limits(model, &[service_id], horizon_s, events, battery)
}

/// Same policy over several services, repaired in the provided order against
/// one shared inventory. The first service that cannot be repaired stops the
/// run.
pub fn simulate_multi_service_longevity_with_limits(
    model: &Model,
    service_ids: &[&str],
    horizon_s: f64,
    events: &[FailureEvent],
    battery: Option<BatteryLimits>,
) -> Result<SurvivalReport, EnvelopeError> {
    simulate_multi_service_longevity_with_load(model, service_ids, horizon_s, events, battery, 0.0)
}

/// Same policy with a constant site load in kW. Energy is drained over each
/// interval; if the inventory would go negative, the run stops with
/// `energy_depleted` before the next event.
pub fn simulate_multi_service_longevity_with_load(
    model: &Model,
    service_ids: &[&str],
    horizon_s: f64,
    events: &[FailureEvent],
    battery: Option<BatteryLimits>,
    load_kw: f64,
) -> Result<SurvivalReport, EnvelopeError> {
    let mut services = Vec::new();
    for service_id in service_ids {
        let Some(service) = model.services.get(*service_id) else {
            return Err(EnvelopeError(format!("unknown service {service_id}")));
        };
        services.push(service);
    }
    let service_label = services.iter().map(|service| service.id.as_str()).collect::<Vec<_>>().join(",");
    if let Some(battery) = battery {
        let initial_energy = initial_inventory(model).get("electric-energy").copied().unwrap_or(0.0);
        let capacity_j = battery.capacity_kwh * 3_600_000.0;
        if initial_energy > capacity_j + 1e-9 {
            return Err(EnvelopeError(format!(
                "initial electric-energy {initial_energy:.0} J exceeds battery capacity {capacity_j:.0} J"
            )));
        }
    }
    let mut current = model.clone();
    let mut inventory = initial_inventory(model);
    let mut log = Vec::new();
    let mut now = 0.0;

    match restore_services(&mut current, &services, 0.0, horizon_s, &mut inventory) {
        Ok((repaired, _repair_s)) => log.push(SurvivalLogEntry {
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
                service: service_label.clone(),
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
        if let Err(offset) = apply_energy_interval(&current, &mut inventory, at - now, battery, load_kw) {
            let failed_at = now + offset;
            log.push(SurvivalLogEntry {
                at_s: failed_at,
                failed: None,
                repaired: Vec::new(),
                energy_after: energy(&inventory),
                blocked_by: Some("energy_depleted".into()),
            });
            return Ok(SurvivalReport {
                service: service_label.clone(),
                horizon_s,
                survived_s: failed_at,
                log,
                final_inventory: inventory,
            });
        }
        now = at;
        if let Some(asset) = current.assets.get_mut(&event.asset) {
            asset.status = AssetStatus::Failed;
        } else {
            return Err(EnvelopeError(format!("unknown failed asset {}", event.asset)));
        }
        match restore_services(&mut current, &services, now, horizon_s, &mut inventory) {
            Ok((repaired, _repair_s)) => log.push(SurvivalLogEntry {
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
                    service: service_label.clone(),
                    horizon_s,
                    survived_s: now,
                    log,
                    final_inventory: inventory,
                });
            }
        }
    }
    if let Err(offset) = apply_energy_interval(&current, &mut inventory, horizon_s - now, battery, load_kw) {
        let failed_at = now + offset;
        log.push(SurvivalLogEntry {
            at_s: failed_at,
            failed: None,
            repaired: Vec::new(),
            energy_after: energy(&inventory),
            blocked_by: Some("energy_depleted".into()),
        });
        return Ok(SurvivalReport {
            service: service_label.clone(),
            horizon_s,
            survived_s: failed_at,
            log,
            final_inventory: inventory,
        });
    }
    Ok(SurvivalReport {
        service: service_label.clone(),
        horizon_s,
        survived_s: horizon_s,
        log,
        final_inventory: inventory,
    })
}

/// S3: like `simulate_multi_service_longevity_with_load`, but a repair's
/// duration advances the simulation clock before the next event is processed.
pub fn simulate_multi_service_longevity_event_driven(
    model: &Model,
    service_ids: &[&str],
    horizon_s: f64,
    events: &[FailureEvent],
    battery: Option<BatteryLimits>,
    load_kw: f64,
) -> Result<SurvivalReport, EnvelopeError> {
    let mut services = Vec::new();
    for service_id in service_ids {
        let Some(service) = model.services.get(*service_id) else {
            return Err(EnvelopeError(format!("unknown service {service_id}")));
        };
        services.push(service);
    }
    let service_label = services.iter().map(|service| service.id.as_str()).collect::<Vec<_>>().join(",");
    if let Some(battery) = battery {
        let initial_energy = initial_inventory(model).get("electric-energy").copied().unwrap_or(0.0);
        let capacity_j = battery.capacity_kwh * 3_600_000.0;
        if initial_energy > capacity_j + 1e-9 {
            return Err(EnvelopeError(format!(
                "initial electric-energy {initial_energy:.0} J exceeds battery capacity {capacity_j:.0} J"
            )));
        }
    }
    let mut current = model.clone();
    let mut inventory = initial_inventory(model);
    let mut log = Vec::new();
    let mut now = 0.0;
    let (repaired, repair_s) = match restore_services(&mut current, &services, 0.0, horizon_s, &mut inventory) {
        Ok(result) => result,
        Err(reason) => {
            log.push(SurvivalLogEntry {
                at_s: 0.0,
                failed: None,
                repaired: Vec::new(),
                energy_after: energy(&inventory),
                blocked_by: Some(reason),
            });
            return Ok(SurvivalReport {
                service: service_label.clone(),
                horizon_s,
                survived_s: 0.0,
                log,
                final_inventory: inventory,
            });
        }
    };
    let repair_s = repair_s.min((horizon_s - now).max(0.0));
    if let Err(offset) = apply_energy_interval(&current, &mut inventory, repair_s, battery, load_kw) {
        let failed_at = now + offset;
        log.push(SurvivalLogEntry {
            at_s: failed_at,
            failed: None,
            repaired: Vec::new(),
            energy_after: energy(&inventory),
            blocked_by: Some("energy_depleted".into()),
        });
        return Ok(SurvivalReport {
            service: service_label.clone(),
            horizon_s,
            survived_s: failed_at,
            log,
            final_inventory: inventory,
        });
    }
    now += repair_s;
    log.push(SurvivalLogEntry {
        at_s: 0.0,
        failed: None,
        repaired,
        energy_after: energy(&inventory),
        blocked_by: None,
    });

    let mut ordered = events.to_vec();
    ordered.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    for event in ordered {
        let at = event.at_s.clamp(now, horizon_s);
        if let Err(offset) = apply_energy_interval(&current, &mut inventory, at - now, battery, load_kw) {
            let failed_at = now + offset;
            log.push(SurvivalLogEntry {
                at_s: failed_at,
                failed: None,
                repaired: Vec::new(),
                energy_after: energy(&inventory),
                blocked_by: Some("energy_depleted".into()),
            });
            return Ok(SurvivalReport {
                service: service_label.clone(),
                horizon_s,
                survived_s: failed_at,
                log,
                final_inventory: inventory,
            });
        }
        now = at;
        if let Some(asset) = current.assets.get_mut(&event.asset) {
            asset.status = AssetStatus::Failed;
        } else {
            return Err(EnvelopeError(format!("unknown failed asset {}", event.asset)));
        }
        let (repaired, repair_s) = match restore_services(&mut current, &services, now, horizon_s, &mut inventory) {
            Ok(result) => result,
            Err(reason) => {
                log.push(SurvivalLogEntry {
                    at_s: now,
                    failed: Some(event.asset),
                    repaired: Vec::new(),
                    energy_after: energy(&inventory),
                    blocked_by: Some(reason),
                });
                return Ok(SurvivalReport {
                    service: service_label.clone(),
                    horizon_s,
                    survived_s: now,
                    log,
                    final_inventory: inventory,
                });
            }
        };
        let repair_s = repair_s.min((horizon_s - now).max(0.0));
        if let Err(offset) = apply_energy_interval(&current, &mut inventory, repair_s, battery, load_kw) {
            let failed_at = now + offset;
            log.push(SurvivalLogEntry {
                at_s: failed_at,
                failed: Some(event.asset),
                repaired: Vec::new(),
                energy_after: energy(&inventory),
                blocked_by: Some("energy_depleted".into()),
            });
            return Ok(SurvivalReport {
                service: service_label.clone(),
                horizon_s,
                survived_s: failed_at,
                log,
                final_inventory: inventory,
            });
        }
        now += repair_s;
        log.push(SurvivalLogEntry {
            at_s: at,
            failed: Some(event.asset),
            repaired,
            energy_after: energy(&inventory),
            blocked_by: None,
        });
    }
    if let Err(offset) = apply_energy_interval(&current, &mut inventory, horizon_s - now, battery, load_kw) {
        let failed_at = now + offset;
        log.push(SurvivalLogEntry {
            at_s: failed_at,
            failed: None,
            repaired: Vec::new(),
            energy_after: energy(&inventory),
            blocked_by: Some("energy_depleted".into()),
        });
        return Ok(SurvivalReport {
            service: service_label.clone(),
            horizon_s,
            survived_s: failed_at,
            log,
            final_inventory: inventory,
        });
    }
    Ok(SurvivalReport { service: service_label, horizon_s, survived_s: horizon_s, log, final_inventory: inventory })
}

/// One external resupply delivery at an absolute time.
#[derive(Clone, Debug, PartialEq)]
pub struct Resupply {
    pub at_s: f64,
    pub resource: String,
    pub si: f64,
}

/// S2: like `simulate_multi_service_longevity_with_load`, plus discrete external
/// resupply deliveries interleaved with failures.
pub fn simulate_multi_service_longevity_with_resupply(
    model: &Model,
    service_ids: &[&str],
    horizon_s: f64,
    events: &[FailureEvent],
    resupplies: &[Resupply],
    battery: Option<BatteryLimits>,
    load_kw: f64,
) -> Result<SurvivalReport, EnvelopeError> {
    let mut services = Vec::new();
    for service_id in service_ids {
        let Some(service) = model.services.get(*service_id) else {
            return Err(EnvelopeError(format!("unknown service {service_id}")));
        };
        services.push(service);
    }
    let service_label = services.iter().map(|service| service.id.as_str()).collect::<Vec<_>>().join(",");
    let mut current = model.clone();
    let mut inventory = initial_inventory(model);
    let mut log = Vec::new();
    let mut now = 0.0;
    match restore_services(&mut current, &services, 0.0, horizon_s, &mut inventory) {
        Ok((repaired, _repair_s)) => log.push(SurvivalLogEntry {
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
                service: service_label.clone(),
                horizon_s,
                survived_s: 0.0,
                log,
                final_inventory: inventory,
            });
        }
    }

    enum Item<'a> {
        Failure(&'a FailureEvent),
        Supply(&'a Resupply),
    }
    let mut timeline: Vec<(f64, Item)> = Vec::new();
    for event in events {
        timeline.push((event.at_s, Item::Failure(event)));
    }
    for supply in resupplies {
        timeline.push((supply.at_s, Item::Supply(supply)));
    }
    timeline.sort_by(|left, right| {
        left.0
            .total_cmp(&right.0)
            .then_with(|| matches!(left.1, Item::Supply(_)).cmp(&matches!(right.1, Item::Supply(_))).reverse())
    });
    for (at_s, item) in timeline {
        let at = at_s.clamp(now, horizon_s);
        if let Err(offset) = apply_energy_interval(&current, &mut inventory, at - now, battery, load_kw) {
            let failed_at = now + offset;
            log.push(SurvivalLogEntry {
                at_s: failed_at,
                failed: None,
                repaired: Vec::new(),
                energy_after: energy(&inventory),
                blocked_by: Some("energy_depleted".into()),
            });
            return Ok(SurvivalReport {
                service: service_label.clone(),
                horizon_s,
                survived_s: failed_at,
                log,
                final_inventory: inventory,
            });
        }
        now = at;
        match item {
            Item::Supply(supply) => {
                *inventory.entry(supply.resource.clone()).or_insert(0.0) += supply.si;
                log.push(SurvivalLogEntry {
                    at_s: now,
                    failed: None,
                    repaired: Vec::new(),
                    energy_after: energy(&inventory),
                    blocked_by: None,
                });
            }
            Item::Failure(event) => {
                if let Some(asset) = current.assets.get_mut(&event.asset) {
                    asset.status = AssetStatus::Failed;
                } else {
                    return Err(EnvelopeError(format!("unknown failed asset {}", event.asset)));
                }
                match restore_services(&mut current, &services, now, horizon_s, &mut inventory) {
                    Ok((repaired, _repair_s)) => log.push(SurvivalLogEntry {
                        at_s: now,
                        failed: Some(event.asset.clone()),
                        repaired,
                        energy_after: energy(&inventory),
                        blocked_by: None,
                    }),
                    Err(reason) => {
                        log.push(SurvivalLogEntry {
                            at_s: now,
                            failed: Some(event.asset.clone()),
                            repaired: Vec::new(),
                            energy_after: energy(&inventory),
                            blocked_by: Some(reason),
                        });
                        return Ok(SurvivalReport {
                            service: service_label.clone(),
                            horizon_s,
                            survived_s: now,
                            log,
                            final_inventory: inventory,
                        });
                    }
                }
            }
        }
    }
    if let Err(offset) = apply_energy_interval(&current, &mut inventory, horizon_s - now, battery, load_kw) {
        let failed_at = now + offset;
        log.push(SurvivalLogEntry {
            at_s: failed_at,
            failed: None,
            repaired: Vec::new(),
            energy_after: energy(&inventory),
            blocked_by: Some("energy_depleted".into()),
        });
        return Ok(SurvivalReport {
            service: service_label.clone(),
            horizon_s,
            survived_s: failed_at,
            log,
            final_inventory: inventory,
        });
    }
    Ok(SurvivalReport { service: service_label, horizon_s, survived_s: horizon_s, log, final_inventory: inventory })
}

fn restore_services(
    model: &mut Model,
    services: &[&Service],
    now_s: f64,
    horizon_s: f64,
    inventory: &mut BTreeMap<String, f64>,
) -> Result<(Vec<String>, f64), String> {
    let mut repaired = Vec::new();
    let mut total_s = 0.0;
    for service in services {
        let (next_repaired, next_s) = restore_service(model, service, now_s, horizon_s, inventory)?;
        repaired.extend(next_repaired);
        total_s += next_s;
    }
    Ok((repaired, total_s))
}

fn restore_service(
    model: &mut Model,
    service: &Service,
    now_s: f64,
    horizon_s: f64,
    inventory: &mut BTreeMap<String, f64>,
) -> Result<(Vec<String>, f64), String> {
    let remaining = (horizon_s - now_s).max(0.0);
    let mut repaired = Vec::new();
    let mut total_s = 0.0;
    for asset in &service.requires {
        if model.assets.get(asset).is_some_and(|item| item.status == AssetStatus::Available) {
            continue;
        }
        total_s += repair_one(model, asset, remaining, inventory, &mut repaired)?;
    }
    if !service.requires_any.is_empty() {
        let already_available = service
            .requires_any
            .iter()
            .any(|asset| model.assets.get(asset).is_some_and(|item| item.status == AssetStatus::Available));
        if !already_available {
            let mut best: Option<String> = None;
            let mut alternative_s = 0.0;
            for asset in &service.requires_any {
                let snapshot = (model.clone(), inventory.clone());
                match repair_one(model, asset, remaining, inventory, &mut Vec::new()) {
                    Ok(duration_s) => {
                        best = Some(asset.clone());
                        alternative_s = duration_s;
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
                Some(asset) if !asset.starts_with("blocked:") => {
                    repaired.push(asset);
                    total_s += alternative_s;
                }
                _ => return Err(best.unwrap_or_else(|| "no alternative service asset".into())),
            }
        }
    }
    Ok((repaired, total_s))
}

fn repair_one(
    model: &mut Model,
    asset: &str,
    remaining_s: f64,
    inventory: &mut BTreeMap<String, f64>,
    repaired: &mut Vec<String>,
) -> Result<f64, String> {
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
            Ok(plan.total_duration_s)
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

fn recharge_j_per_s(model: &Model) -> Option<f64> {
    let process = model.processes.iter().find(|process| process.id == "recharge-energy")?;
    let duration = process.duration_s?;
    let output = process.outputs.iter().find(|output| output.resource == "electric-energy")?;
    if duration <= 0.0 {
        return None;
    }
    Some(output.si / duration)
}

fn apply_energy_interval(
    model: &Model,
    inventory: &mut BTreeMap<String, f64>,
    elapsed_s: f64,
    battery: Option<BatteryLimits>,
    load_kw: f64,
) -> Result<(), f64> {
    if elapsed_s <= 0.0 {
        return Ok(());
    }
    let energy = inventory.entry("electric-energy".into()).or_insert(0.0);
    let mut net_j_per_s = 0.0;
    if model.assets.get("pv-1").is_some_and(|asset| asset.status == AssetStatus::Available) {
        if let Some(raw_j_per_s) = recharge_j_per_s(model) {
            net_j_per_s += match battery {
                Some(battery) => raw_j_per_s.min(battery.charge_kw * 1000.0),
                None => raw_j_per_s,
            };
        }
    }
    net_j_per_s -= load_kw.max(0.0) * 1000.0;
    if net_j_per_s < 0.0 {
        let time_to_empty = *energy / -net_j_per_s;
        if time_to_empty < elapsed_s {
            *energy = 0.0;
            return Err(time_to_empty);
        }
    }
    *energy += net_j_per_s * elapsed_s;
    if let Some(battery) = battery {
        let capacity_j = battery.capacity_kwh * 3_600_000.0;
        if *energy > capacity_j {
            *energy = capacity_j;
        }
    }
    if *energy < 0.0 {
        *energy = 0.0;
        Err(0.0)
    } else {
        Ok(())
    }
}

#[allow(dead_code)]
fn add_recharge(model: &Model, inventory: &mut BTreeMap<String, f64>, elapsed_s: f64, battery: Option<BatteryLimits>) {
    if elapsed_s <= 0.0 {
        return;
    }
    if !model.assets.get("pv-1").is_some_and(|asset| asset.status == AssetStatus::Available) {
        return;
    }
    let Some(raw_j_per_s) = recharge_j_per_s(model) else {
        return;
    };
    let added_j = match battery {
        Some(battery) => raw_j_per_s.min(battery.charge_kw * 1000.0) * elapsed_s,
        None => raw_j_per_s * elapsed_s,
    };
    let energy = inventory.entry("electric-energy".into()).or_insert(0.0);
    *energy += added_j;
    if let Some(battery) = battery {
        let capacity_j = battery.capacity_kwh * 3_600_000.0;
        if *energy > capacity_j {
            *energy = capacity_j;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApprovalDecision {
    /// The proposal is still waiting for an operator; do not execute.
    Required,
    Approved,
    Rejected,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecoveryPolicy {
    pub decision: ApprovalDecision,
    /// Maximum age of the approval at execution time. A plan whose duration
    /// exceeds the lease is not applied.
    pub lease_s: f64,
    /// If set, the approved plan digest must match the proposed plan.
    pub approved_plan_digest: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RecoveryReceipt {
    pub at_s: f64,
    pub failed: Option<String>,
    pub repaired: Vec<String>,
    pub plan_digests: Vec<String>,
    pub inventory_before: BTreeMap<String, f64>,
    pub inventory_after: BTreeMap<String, f64>,
    pub verified: bool,
    pub digest: String,
}

#[derive(Serialize)]
struct ReceiptBody<'a> {
    at_s: f64,
    failed: Option<&'a str>,
    repaired: &'a [String],
    plan_digests: &'a [String],
    inventory_before: &'a BTreeMap<String, f64>,
    inventory_after: &'a BTreeMap<String, f64>,
    verified: bool,
}

impl RecoveryReceipt {
    /// Recompute and compare the receipt digest. A false result means the
    /// receipt body was altered after issuance.
    pub fn verify(&self) -> bool {
        let body = ReceiptBody {
            at_s: self.at_s,
            failed: self.failed.as_deref(),
            repaired: &self.repaired,
            plan_digests: &self.plan_digests,
            inventory_before: &self.inventory_before,
            inventory_after: &self.inventory_after,
            verified: self.verified,
        };
        sha256_of("poie.recovery.receipt/1", &body).is_ok_and(|digest| digest == self.digest)
    }
}

fn make_receipt(
    at_s: f64,
    failed: Option<String>,
    repaired: Vec<String>,
    plan_digests: Vec<String>,
    inventory_before: BTreeMap<String, f64>,
    inventory_after: BTreeMap<String, f64>,
    verified: bool,
) -> RecoveryReceipt {
    let body = ReceiptBody {
        at_s,
        failed: failed.as_deref(),
        repaired: &repaired,
        plan_digests: &plan_digests,
        inventory_before: &inventory_before,
        inventory_after: &inventory_after,
        verified,
    };
    let digest = sha256_of("poie.recovery.receipt/1", &body).unwrap_or_default();
    RecoveryReceipt { at_s, failed, repaired, plan_digests, inventory_before, inventory_after, verified, digest }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecoveryLogEntry {
    pub at_s: f64,
    pub failed: Option<String>,
    pub repaired: Vec<String>,
    pub approved: bool,
    pub approval: Option<String>,
    pub plan_digests: Vec<String>,
    pub receipt: Option<RecoveryReceipt>,
    pub verified: bool,
    pub blocked_by: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecoveryReport {
    pub service: String,
    pub horizon_s: f64,
    pub survived_s: f64,
    pub log: Vec<RecoveryLogEntry>,
}

/// S3: a closed recovery loop with an explicit approval state and lease.
///
/// A proposal is created only by the deterministic S1 closure. `Required` and
/// `Rejected` both stop the run without applying the plan; an `Approved` policy
/// applies the plan only if its duration fits within `lease_s`. After applying,
/// the service requirements are re-verified. This is an isolated simulation,
/// not a hardware authority path.
pub fn simulate_recovery(
    model: &Model,
    service_id: &str,
    horizon_s: f64,
    events: &[FailureEvent],
    approved: bool,
) -> Result<RecoveryReport, EnvelopeError> {
    let policy = RecoveryPolicy {
        decision: if approved { ApprovalDecision::Approved } else { ApprovalDecision::Required },
        lease_s: f64::INFINITY,
        approved_plan_digest: None,
    };
    simulate_recovery_with_policy(model, service_id, horizon_s, events, policy)
}

pub fn simulate_recovery_with_policy(
    model: &Model,
    service_id: &str,
    horizon_s: f64,
    events: &[FailureEvent],
    policy: RecoveryPolicy,
) -> Result<RecoveryReport, EnvelopeError> {
    simulate_recovery_inner(model, service_id, horizon_s, events, policy, None, None)
}

/// S3: recovery with a revocation deadline. Once `revoke_at_s` is reached, no
/// further repair is applied; the loop records `approval_revoked` and stops.
pub fn simulate_recovery_with_revocation(
    model: &Model,
    service_id: &str,
    horizon_s: f64,
    events: &[FailureEvent],
    policy: RecoveryPolicy,
    revoke_at_s: f64,
) -> Result<RecoveryReport, EnvelopeError> {
    simulate_recovery_inner(model, service_id, horizon_s, events, policy, Some(revoke_at_s), None)
}

/// S3: recovery with a denial at one event time.
///
/// Repairs before the denial remain applied; the denial stops the run at that
/// event with `approval_denied_at_event`.
pub fn simulate_recovery_with_denial(
    model: &Model,
    service_id: &str,
    horizon_s: f64,
    events: &[FailureEvent],
    policy: RecoveryPolicy,
    deny_at_s: f64,
) -> Result<RecoveryReport, EnvelopeError> {
    simulate_recovery_inner(model, service_id, horizon_s, events, policy, None, Some(deny_at_s))
}

fn simulate_recovery_inner(
    model: &Model,
    service_id: &str,
    horizon_s: f64,
    events: &[FailureEvent],
    policy: RecoveryPolicy,
    revoke_at_s: Option<f64>,
    deny_at_s: Option<f64>,
) -> Result<RecoveryReport, EnvelopeError> {
    let Some(service) = model.services.get(service_id) else {
        return Err(EnvelopeError(format!("unknown service {service_id}")));
    };
    let mut current = model.clone();
    let mut inventory = initial_inventory(model);
    let mut log = Vec::new();
    let mut now = 0.0;
    let approval = approval_label(policy.decision);

    let initial_before = inventory.clone();
    if revoke_at_s.is_some_and(|revoke| now >= revoke) {
        log.push(RecoveryLogEntry {
            at_s: now,
            failed: None,
            repaired: Vec::new(),
            approved: false,
            approval: Some(approval.clone()),
            plan_digests: Vec::new(),
            receipt: None,
            verified: false,
            blocked_by: Some("approval_revoked".into()),
        });
        return Ok(RecoveryReport { service: service.id.clone(), horizon_s, survived_s: now, log });
    }
    match recover_service(&mut current, service, now, horizon_s, &mut inventory, policy.clone(), revoke_at_s) {
        Ok((repaired, plan_digests)) => log.push(RecoveryLogEntry {
            at_s: 0.0,
            failed: None,
            repaired: repaired.clone(),
            approved: policy.decision == ApprovalDecision::Approved,
            approval: Some(approval.clone()),
            plan_digests: plan_digests.clone(),
            receipt: Some(make_receipt(
                0.0,
                None,
                repaired.clone(),
                plan_digests,
                initial_before.clone(),
                inventory.clone(),
                true,
            )),
            verified: true,
            blocked_by: None,
        }),
        Err(reason) => {
            log.push(RecoveryLogEntry {
                at_s: 0.0,
                failed: None,
                repaired: Vec::new(),
                approved: false,
                approval: Some(approval),
                plan_digests: Vec::new(),
                receipt: None,
                verified: false,
                blocked_by: Some(reason),
            });
            return Ok(RecoveryReport { service: service.id.clone(), horizon_s, survived_s: 0.0, log });
        }
    }

    let mut ordered = events.to_vec();
    ordered.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    for event in ordered {
        let at = event.at_s.clamp(now, horizon_s);
        now = at;
        if let Some(asset) = current.assets.get_mut(&event.asset) {
            asset.status = AssetStatus::Failed;
        } else {
            return Err(EnvelopeError(format!("unknown failed asset {}", event.asset)));
        }
        let before = inventory.clone();
        if revoke_at_s.is_some_and(|revoke| now >= revoke) {
            log.push(RecoveryLogEntry {
                at_s: now,
                failed: Some(event.asset.clone()),
                repaired: Vec::new(),
                approved: false,
                approval: Some(approval.clone()),
                plan_digests: Vec::new(),
                receipt: None,
                verified: false,
                blocked_by: Some("approval_revoked".into()),
            });
            return Ok(RecoveryReport { service: service.id.clone(), horizon_s, survived_s: now, log });
        }
        if deny_at_s.is_some_and(|deny| (now - deny).abs() < 1e-9) {
            log.push(RecoveryLogEntry {
                at_s: now,
                failed: Some(event.asset.clone()),
                repaired: Vec::new(),
                approved: false,
                approval: Some(approval.clone()),
                plan_digests: Vec::new(),
                receipt: None,
                verified: false,
                blocked_by: Some(format!("approval_denied_at_event:{}", event.asset)),
            });
            return Ok(RecoveryReport { service: service.id.clone(), horizon_s, survived_s: now, log });
        }
        match recover_service(&mut current, service, now, horizon_s, &mut inventory, policy.clone(), revoke_at_s) {
            Ok((repaired, plan_digests)) => log.push(RecoveryLogEntry {
                at_s: now,
                failed: Some(event.asset.clone()),
                repaired: repaired.clone(),
                approved: policy.decision == ApprovalDecision::Approved,
                approval: Some(approval.clone()),
                plan_digests: plan_digests.clone(),
                receipt: Some(make_receipt(
                    now,
                    Some(event.asset.clone()),
                    repaired.clone(),
                    plan_digests,
                    before.clone(),
                    inventory.clone(),
                    true,
                )),
                verified: true,
                blocked_by: None,
            }),
            Err(reason) => {
                log.push(RecoveryLogEntry {
                    at_s: now,
                    failed: Some(event.asset),
                    repaired: Vec::new(),
                    approved: false,
                    approval: Some(approval),
                    plan_digests: Vec::new(),
                    receipt: None,
                    verified: false,
                    blocked_by: Some(reason),
                });
                return Ok(RecoveryReport { service: service.id.clone(), horizon_s, survived_s: now, log });
            }
        }
    }
    Ok(RecoveryReport { service: service.id.clone(), horizon_s, survived_s: horizon_s, log })
}

#[derive(Clone, Debug, PartialEq)]
pub struct MultiServiceRecoveryReport {
    pub services: Vec<String>,
    pub horizon_s: f64,
    pub survived_s: f64,
    pub log: Vec<RecoveryLogEntry>,
}

/// S3: approval-gated recovery loop across several services with one shared
/// inventory. All services must stay verified after each repair event.
pub fn simulate_multi_service_recovery_with_policy(
    model: &Model,
    service_ids: &[&str],
    horizon_s: f64,
    events: &[FailureEvent],
    policy: RecoveryPolicy,
) -> Result<MultiServiceRecoveryReport, EnvelopeError> {
    simulate_multi_service_recovery_inner(model, service_ids, horizon_s, events, policy, None)
}

/// S3: multi-service recovery with a revocation deadline. Once revoked, no
/// further service repair is applied.
pub fn simulate_multi_service_recovery_with_revocation(
    model: &Model,
    service_ids: &[&str],
    horizon_s: f64,
    events: &[FailureEvent],
    policy: RecoveryPolicy,
    revoke_at_s: f64,
) -> Result<MultiServiceRecoveryReport, EnvelopeError> {
    simulate_multi_service_recovery_inner(model, service_ids, horizon_s, events, policy, Some(revoke_at_s))
}

fn simulate_multi_service_recovery_inner(
    model: &Model,
    service_ids: &[&str],
    horizon_s: f64,
    events: &[FailureEvent],
    policy: RecoveryPolicy,
    revoke_at_s: Option<f64>,
) -> Result<MultiServiceRecoveryReport, EnvelopeError> {
    let mut services = Vec::new();
    for service_id in service_ids {
        let Some(service) = model.services.get(*service_id) else {
            return Err(EnvelopeError(format!("unknown service {service_id}")));
        };
        services.push(service.clone());
    }
    let mut current = model.clone();
    let mut inventory = initial_inventory(model);
    let mut log = Vec::new();
    let approval = approval_label(policy.decision);

    let initial_before = inventory.clone();
    if revoke_at_s.is_some_and(|revoke| 0.0 >= revoke) {
        log.push(RecoveryLogEntry {
            at_s: 0.0,
            failed: None,
            repaired: Vec::new(),
            approved: false,
            approval: Some(approval.clone()),
            plan_digests: Vec::new(),
            receipt: None,
            verified: false,
            blocked_by: Some("approval_revoked".into()),
        });
        return Ok(MultiServiceRecoveryReport {
            services: services.iter().map(|item| item.id.clone()).collect(),
            horizon_s,
            survived_s: 0.0,
            log,
        });
    }
    let mut repaired = Vec::new();
    let mut digests = Vec::new();
    for service in &services {
        match recover_service(&mut current, service, 0.0, horizon_s, &mut inventory, policy.clone(), revoke_at_s) {
            Ok((next_repaired, next_digests)) => {
                repaired.extend(next_repaired);
                digests.extend(next_digests);
            }
            Err(reason) => {
                log.push(RecoveryLogEntry {
                    at_s: 0.0,
                    failed: None,
                    repaired: Vec::new(),
                    approved: false,
                    approval: Some(approval.clone()),
                    plan_digests: Vec::new(),
                    receipt: None,
                    verified: false,
                    blocked_by: Some(format!("{}:{reason}", service.id)),
                });
                return Ok(MultiServiceRecoveryReport {
                    services: services.iter().map(|item| item.id.clone()).collect(),
                    horizon_s,
                    survived_s: 0.0,
                    log,
                });
            }
        }
    }
    log.push(RecoveryLogEntry {
        at_s: 0.0,
        failed: None,
        repaired: repaired.clone(),
        approved: policy.decision == ApprovalDecision::Approved,
        approval: Some(approval.clone()),
        plan_digests: digests.clone(),
        receipt: Some(make_receipt(0.0, None, repaired, digests, initial_before, inventory.clone(), true)),
        verified: true,
        blocked_by: None,
    });

    let mut ordered = events.to_vec();
    ordered.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
    let mut now = 0.0;
    for event in ordered {
        now = event.at_s.clamp(now, horizon_s);
        if let Some(asset) = current.assets.get_mut(&event.asset) {
            asset.status = AssetStatus::Failed;
        } else {
            return Err(EnvelopeError(format!("unknown failed asset {}", event.asset)));
        }
        let before = inventory.clone();
        if revoke_at_s.is_some_and(|revoke| now >= revoke) {
            log.push(RecoveryLogEntry {
                at_s: now,
                failed: Some(event.asset.clone()),
                repaired: Vec::new(),
                approved: false,
                approval: Some(approval.clone()),
                plan_digests: Vec::new(),
                receipt: None,
                verified: false,
                blocked_by: Some("approval_revoked".into()),
            });
            return Ok(MultiServiceRecoveryReport {
                services: services.iter().map(|item| item.id.clone()).collect(),
                horizon_s,
                survived_s: now,
                log,
            });
        }
        let mut repaired = Vec::new();
        let mut digests = Vec::new();
        for service in &services {
            match recover_service(&mut current, service, now, horizon_s, &mut inventory, policy.clone(), revoke_at_s) {
                Ok((next_repaired, next_digests)) => {
                    repaired.extend(next_repaired);
                    digests.extend(next_digests);
                }
                Err(reason) => {
                    log.push(RecoveryLogEntry {
                        at_s: now,
                        failed: Some(event.asset.clone()),
                        repaired: Vec::new(),
                        approved: false,
                        approval: Some(approval),
                        plan_digests: Vec::new(),
                        receipt: None,
                        verified: false,
                        blocked_by: Some(format!("{}:{reason}", service.id)),
                    });
                    return Ok(MultiServiceRecoveryReport {
                        services: services.iter().map(|item| item.id.clone()).collect(),
                        horizon_s,
                        survived_s: now,
                        log,
                    });
                }
            }
        }
        log.push(RecoveryLogEntry {
            at_s: now,
            failed: Some(event.asset.clone()),
            repaired: repaired.clone(),
            approved: policy.decision == ApprovalDecision::Approved,
            approval: Some(approval.clone()),
            plan_digests: digests.clone(),
            receipt: Some(make_receipt(now, Some(event.asset), repaired, digests, before, inventory.clone(), true)),
            verified: true,
            blocked_by: None,
        });
    }
    Ok(MultiServiceRecoveryReport {
        services: services.iter().map(|item| item.id.clone()).collect(),
        horizon_s,
        survived_s: horizon_s,
        log,
    })
}

fn approval_label(decision: ApprovalDecision) -> String {
    match decision {
        ApprovalDecision::Required => "required",
        ApprovalDecision::Approved => "approved",
        ApprovalDecision::Rejected => "rejected",
    }
    .to_string()
}

fn recover_service(
    model: &mut Model,
    service: &Service,
    now_s: f64,
    horizon_s: f64,
    inventory: &mut BTreeMap<String, f64>,
    policy: RecoveryPolicy,
    revoke_at_s: Option<f64>,
) -> Result<(Vec<String>, Vec<String>), String> {
    let remaining = (horizon_s - now_s).max(0.0);
    let mut repaired = Vec::new();
    let mut digests = Vec::new();
    for asset in &service.requires {
        if model.assets.get(asset).is_some_and(|item| item.status == AssetStatus::Available) {
            continue;
        }
        let with_inventory = model_with_inventory(model, inventory);
        let plan = match analyze(&with_inventory, asset, remaining) {
            ClosureOutcome::Restorable(plan) => plan,
            ClosureOutcome::Blocked(blocked) => {
                return Err(blocked.reasons.first().map(ToString::to_string).unwrap_or_else(|| "blocked".into()));
            }
        };
        let digest = sha256_of("poie.recovery.plan/1", &plan).ok();
        check_approval(&policy, asset, plan.total_duration_s, digest.as_deref())?;
        if let Some(revoke) = revoke_at_s {
            if revoke > now_s && revoke < now_s + plan.total_duration_s {
                return Err(format!(
                    "approval_revoked_mid_plan:{asset}: revoke {revoke:.0}s during {:.0}s plan",
                    plan.total_duration_s
                ));
            }
        }
        apply_recovery_plan(model, inventory, plan);
        repaired.push(asset.to_string());
        digests.push(digest.unwrap_or_default());
    }
    if !service.requires_any.is_empty()
        && !service
            .requires_any
            .iter()
            .any(|asset| model.assets.get(asset).is_some_and(|item| item.status == AssetStatus::Available))
    {
        let mut applied = false;
        for asset in &service.requires_any {
            let with_inventory = model_with_inventory(model, inventory);
            if let ClosureOutcome::Restorable(plan) = analyze(&with_inventory, asset, remaining) {
                let digest = sha256_of("poie.recovery.plan/1", &plan).ok();
                check_approval(&policy, asset, plan.total_duration_s, digest.as_deref())?;
                if let Some(revoke) = revoke_at_s {
                    if revoke > now_s && revoke < now_s + plan.total_duration_s {
                        return Err(format!(
                            "approval_revoked_mid_plan:{asset}: revoke {revoke:.0}s during {:.0}s plan",
                            plan.total_duration_s
                        ));
                    }
                }
                apply_recovery_plan(model, inventory, plan);
                repaired.push(asset.to_string());
                digests.push(digest.unwrap_or_default());
                applied = true;
                break;
            }
        }
        if !applied {
            return Err("no alternative service asset".into());
        }
    }
    let verified = service
        .requires
        .iter()
        .all(|asset| model.assets.get(asset).is_some_and(|item| item.status == AssetStatus::Available));
    if !verified {
        return Err("post-repair verification failed".into());
    }
    Ok((repaired, digests))
}

fn check_approval(
    policy: &RecoveryPolicy,
    asset: &str,
    plan_duration_s: f64,
    plan_digest: Option<&str>,
) -> Result<(), String> {
    match policy.decision {
        ApprovalDecision::Required => return Err(format!("approval_required:{asset}")),
        ApprovalDecision::Rejected => return Err(format!("approval_rejected:{asset}")),
        ApprovalDecision::Approved => {}
    }
    if plan_duration_s > policy.lease_s {
        return Err(format!(
            "approval_lease_expired:{asset}: need {plan_duration_s:.0}s, lease {:.0}s",
            policy.lease_s
        ));
    }
    if let Some(expected) = &policy.approved_plan_digest {
        if plan_digest != Some(expected.as_str()) {
            return Err(format!("approval_digest_mismatch:{asset}"));
        }
    }
    Ok(())
}

fn apply_recovery_plan(model: &mut Model, inventory: &mut BTreeMap<String, f64>, plan: Plan) {
    for step in &plan.steps {
        if let StepAction::Restore(restored) = &step.action {
            if let Some(asset) = model.assets.get_mut(restored) {
                asset.status = AssetStatus::Available;
            }
        }
    }
    *inventory = plan.remaining_inventory;
}

#[derive(Clone, Debug, PartialEq)]
pub struct JointCoverageCase {
    pub failed: Vec<String>,
    pub independent_recoverable: bool,
    pub joint_recoverable: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct JointCoverageReport {
    pub service: String,
    pub horizon_s: f64,
    pub cases: Vec<JointCoverageCase>,
    pub independent_recoverable: usize,
    pub joint_recoverable: usize,
}

impl JointCoverageReport {
    pub fn total(&self) -> usize {
        self.cases.len()
    }

    pub fn optimistic_gap(&self) -> usize {
        self.cases.iter().filter(|case| case.independent_recoverable && !case.joint_recoverable).count()
    }
}

/// Compare the independent coverage envelope with a joint policy that uses one
/// shared inventory and one serial repair order. Failures in each case are
/// scheduled at t=0 and repaired immediately; the battery limits are optional.
pub fn joint_coverage(
    model: &Model,
    service_id: &str,
    horizon_s: f64,
    max_failures: usize,
    battery: Option<BatteryLimits>,
) -> Result<JointCoverageReport, EnvelopeError> {
    let independent = failure_envelope(model, service_id, horizon_s, max_failures)?;
    let mut cases = Vec::new();
    let mut independent_recoverable = 0;
    let mut joint_recoverable = 0;
    for case in &independent.cases {
        if case.recoverable {
            independent_recoverable += 1;
        }
        let joint = if !case.recoverable {
            false
        } else {
            let events: Vec<FailureEvent> =
                case.failed.iter().map(|asset| FailureEvent { at_s: 0.0, asset: asset.clone() }).collect();
            let report =
                simulate_multi_service_longevity_with_limits(model, &[service_id], horizon_s, &events, battery)?;
            report.log.iter().all(|entry| entry.blocked_by.is_none())
        };
        if joint {
            joint_recoverable += 1;
        }
        cases.push(JointCoverageCase {
            failed: case.failed.clone(),
            independent_recoverable: case.recoverable,
            joint_recoverable: joint,
        });
    }
    Ok(JointCoverageReport {
        service: independent.service,
        horizon_s,
        cases,
        independent_recoverable,
        joint_recoverable,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrderCoverageCase {
    pub failed: Vec<String>,
    pub standard_joint: bool,
    pub best_joint: bool,
    pub best_order: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrderCoverageReport {
    pub total: usize,
    pub standard_joint: usize,
    pub best_joint: usize,
    pub improved: Vec<OrderCoverageCase>,
}

impl OrderCoverageReport {
    pub fn order_sensitive(&self) -> usize {
        self.improved.len()
    }
}

fn permutations(items: &[String]) -> Vec<Vec<String>> {
    if items.len() <= 1 {
        return vec![items.to_vec()];
    }
    let mut result = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let mut rest = items.to_vec();
        rest.remove(index);
        for mut tail in permutations(&rest) {
            let mut next = vec![item.clone()];
            next.append(&mut tail);
            result.push(next);
        }
    }
    result
}

/// S2: compare the fixed failure order with the best repair order per case.
pub fn joint_coverage_with_order_search(
    model: &Model,
    service_id: &str,
    horizon_s: f64,
    max_failures: usize,
    battery: Option<BatteryLimits>,
) -> Result<OrderCoverageReport, EnvelopeError> {
    let independent = failure_envelope(model, service_id, horizon_s, max_failures)?;
    let mut standard_joint = 0;
    let mut best_joint = 0;
    let mut improved = Vec::new();
    for case in &independent.cases {
        if !case.recoverable {
            continue;
        }
        let standard_events: Vec<FailureEvent> =
            case.failed.iter().map(|asset| FailureEvent { at_s: 0.0, asset: asset.clone() }).collect();
        let standard =
            simulate_multi_service_longevity_with_limits(model, &[service_id], horizon_s, &standard_events, battery)?
                .log
                .iter()
                .all(|entry| entry.blocked_by.is_none());
        if standard {
            standard_joint += 1;
        }
        let mut best = false;
        let mut best_order = None;
        for order in permutations(&case.failed) {
            let events: Vec<FailureEvent> =
                order.iter().map(|asset| FailureEvent { at_s: 0.0, asset: asset.clone() }).collect();
            let recovered =
                simulate_multi_service_longevity_with_limits(model, &[service_id], horizon_s, &events, battery)?
                    .log
                    .iter()
                    .all(|entry| entry.blocked_by.is_none());
            if recovered {
                best = true;
                if !standard {
                    best_order = Some(order.clone());
                }
                break;
            }
        }
        if best {
            best_joint += 1;
        }
        if best && !standard {
            improved.push(OrderCoverageCase {
                failed: case.failed.clone(),
                standard_joint: standard,
                best_joint: best,
                best_order,
            });
        }
    }
    Ok(OrderCoverageReport {
        total: independent.cases.iter().filter(|case| case.recoverable).count(),
        standard_joint,
        best_joint,
        improved,
    })
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
    fn base06_three_failure_order_search_still_has_no_order_sensitivity() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let model = Model::load(&root.join("scenarios/base-06/model.toml")).expect("Base-06");
        let report = joint_coverage_with_order_search(
            &model,
            "potable-water",
            12.0 * 3600.0,
            3,
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        )
        .unwrap();
        assert_eq!(report.total, 44);
        assert_eq!(report.standard_joint, 40);
        assert_eq!(report.best_joint, 40);
        assert_eq!(report.order_sensitive(), 0);
    }

    #[test]
    fn base06_order_search_does_not_close_the_joint_gap() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let model = Model::load(&root.join("scenarios/base-06/model.toml")).expect("Base-06");
        let report = joint_coverage_with_order_search(
            &model,
            "potable-water",
            12.0 * 3600.0,
            2,
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        )
        .unwrap();
        assert_eq!(report.total, 25);
        assert_eq!(report.standard_joint, 24);
        assert_eq!(report.best_joint, 24);
        assert_eq!(report.order_sensitive(), 0);
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
    fn battery_limits_cap_energy_after_idle_recharge() {
        let report = simulate_longevity_with_limits(
            &base05(),
            "potable-water",
            30.0 * 86400.0,
            &[],
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        )
        .unwrap();
        let energy = report.final_inventory.get("electric-energy").copied().unwrap_or(0.0);
        assert!(energy <= 200.0 * 3_600_000.0 + 1.0);
    }

    #[test]
    fn multi_service_repairs_a_failed_lathe() {
        let report = simulate_multi_service_longevity_with_limits(
            &base05(),
            &["potable-water", "workshop-operability"],
            30.0 * 86400.0,
            &[FailureEvent { at_s: 10.0 * 86400.0, asset: "lathe-1".into() }],
            None,
        )
        .unwrap();
        let entry = report.log.iter().find(|entry| entry.failed.as_deref() == Some("lathe-1")).expect("lathe event");
        assert!(entry.repaired.iter().any(|asset| asset == "lathe-1"));
        assert!(entry.blocked_by.is_none());
    }

    #[test]
    fn recovery_receipt_verifies_and_detects_tampering() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let model = Model::load(&root.join("scenarios/base-05/model.toml")).expect("Base-05");
        let policy =
            RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 8.0 * 3600.0, approved_plan_digest: None };
        let report = simulate_recovery_with_policy(
            &model,
            "potable-water",
            30.0 * 86400.0,
            &[FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p1".into() }],
            policy,
        )
        .unwrap();
        let receipts: Vec<_> = report.log.iter().filter_map(|entry| entry.receipt.clone()).collect();
        assert_eq!(receipts.len(), 2);
        assert!(receipts.iter().all(|receipt| receipt.verify()));
        assert_eq!(receipts[0].inventory_after, receipts[1].inventory_before);
        let mut tampered = receipts[0].clone();
        tampered.verified = false;
        assert!(!tampered.verify());
    }

    #[test]
    fn recovery_rejects_a_mismatched_plan_digest() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let model = Model::load(&root.join("scenarios/base-05/model.toml")).expect("Base-05");
        let policy =
            RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 8.0 * 3600.0, approved_plan_digest: None };
        let report = simulate_recovery_with_policy(&model, "potable-water", 30.0 * 86400.0, &[], policy).unwrap();
        let correct = report.log[0].plan_digests[0].clone();
        assert_eq!(correct.len(), 64);
        let wrong = RecoveryPolicy {
            decision: ApprovalDecision::Approved,
            lease_s: 8.0 * 3600.0,
            approved_plan_digest: Some("0".repeat(64)),
        };
        let blocked = simulate_recovery_with_policy(&model, "potable-water", 30.0 * 86400.0, &[], wrong).unwrap();
        assert_eq!(blocked.survived_s, 0.0);
        assert!(blocked.log[0]
            .blocked_by
            .as_deref()
            .is_some_and(|reason| reason.starts_with("approval_digest_mismatch:")));
    }

    #[test]
    fn recovery_rejection_stops_without_side_effects() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let model = Model::load(&root.join("scenarios/base-05/model.toml")).expect("Base-05");
        let report = simulate_recovery_with_policy(
            &model,
            "potable-water",
            30.0 * 86400.0,
            &[],
            RecoveryPolicy { decision: ApprovalDecision::Rejected, lease_s: 4.0 * 3600.0, approved_plan_digest: None },
        )
        .unwrap();
        assert_eq!(report.survived_s, 0.0);
        assert!(report.log[0].blocked_by.as_deref().is_some_and(|reason| reason.starts_with("approval_rejected:")));
    }

    #[test]
    fn recovery_lease_expires_before_a_long_repair() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let model = Model::load(&root.join("scenarios/base-05/model.toml")).expect("Base-05");
        let report = simulate_recovery_with_policy(
            &model,
            "potable-water",
            30.0 * 86400.0,
            &[],
            RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 2.0 * 3600.0, approved_plan_digest: None },
        )
        .unwrap();
        assert_eq!(report.survived_s, 0.0);
        assert!(report.log[0]
            .blocked_by
            .as_deref()
            .is_some_and(|reason| reason.starts_with("approval_lease_expired:")));
    }

    #[test]
    fn recovery_loop_requires_approval() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let model = Model::load(&root.join("scenarios/base-05/model.toml")).expect("Base-05");
        let report = simulate_recovery(&model, "potable-water", 30.0 * 86400.0, &[], false).unwrap();
        assert_eq!(report.survived_s, 0.0);
        assert!(report.log[0].blocked_by.as_deref().is_some_and(|reason| reason.starts_with("approval_required:")));
    }

    #[test]
    fn approved_recovery_loop_verifies_repairs() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let model = Model::load(&root.join("scenarios/base-05/model.toml")).expect("Base-05");
        let report = simulate_recovery(
            &model,
            "potable-water",
            30.0 * 86400.0,
            &[FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p1".into() }],
            true,
        )
        .unwrap();
        assert_eq!(report.survived_s, 30.0 * 86400.0);
        assert!(report.log.iter().all(|entry| entry.verified && entry.blocked_by.is_none()));
    }

    #[test]
    fn lower_recycling_yield_reduces_the_ten_day_horizon() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut low = Model::load(&root.join("scenarios/base-08/model.toml")).expect("Base-08");
        let process = low.processes.iter_mut().find(|process| process.id == "recycle-bronze").expect("recycle-bronze");
        for output in &mut process.outputs {
            if output.resource == "bronze-bar" {
                output.si *= 0.5;
            }
        }
        let make_events = || {
            let mut events: Vec<FailureEvent> = (1..=8)
                .map(|index| FailureEvent { at_s: f64::from(index * 10) * 86400.0, asset: "pump-p1".into() })
                .collect();
            events.push(FailureEvent { at_s: 35.0 * 86400.0, asset: "lathe-1".into() });
            events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
            events
        };
        let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
        let high = Model::load(&root.join("scenarios/base-08/model.toml")).expect("Base-08");
        let normal = simulate_multi_service_longevity_with_limits(
            &high,
            &["potable-water", "workshop-operability"],
            90.0 * 86400.0,
            &make_events(),
            battery,
        )
        .unwrap();
        let reduced = simulate_multi_service_longevity_with_limits(
            &low,
            &["potable-water", "workshop-operability"],
            90.0 * 86400.0,
            &make_events(),
            battery,
        )
        .unwrap();
        assert!(reduced.survived_s <= normal.survived_s);
    }

    #[test]
    fn regeneration_extends_the_ten_day_failure_horizon() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let load = |name: &str| Model::load(&root.join(format!("scenarios/{name}/model.toml"))).expect(name);
        let make_events = || {
            let mut events: Vec<FailureEvent> = (1..=8)
                .map(|index| FailureEvent { at_s: f64::from(index * 10) * 86400.0, asset: "pump-p1".into() })
                .collect();
            events.push(FailureEvent { at_s: 35.0 * 86400.0, asset: "lathe-1".into() });
            events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
            events
        };
        let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
        let base = simulate_multi_service_longevity_with_limits(
            &load("base-05"),
            &["potable-water", "workshop-operability"],
            90.0 * 86400.0,
            &make_events(),
            battery,
        )
        .unwrap();
        let regen = simulate_multi_service_longevity_with_limits(
            &load("base-08"),
            &["potable-water", "workshop-operability"],
            90.0 * 86400.0,
            &make_events(),
            battery,
        )
        .unwrap();
        assert!(regen.survived_s > base.survived_s);
    }

    #[test]
    fn local_smelting_closes_the_joint_coverage_gap() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
        let base06 = Model::load(&root.join("scenarios/base-06/model.toml")).expect("Base-06");
        let base07 = Model::load(&root.join("scenarios/base-07/model.toml")).expect("Base-07");
        let before = joint_coverage(&base06, "potable-water", 24.0 * 3600.0, 2, battery).unwrap();
        let after = joint_coverage(&base07, "potable-water", 24.0 * 3600.0, 2, battery).unwrap();
        assert!(before.optimistic_gap() >= 1);
        assert_eq!(after.optimistic_gap(), 0);
    }

    #[test]
    fn one_extra_sleeve_closes_the_joint_coverage_gap() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut model = Model::load(&root.join("scenarios/base-06/model.toml")).expect("Base-06");
        let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
        let before = joint_coverage(&model, "potable-water", 12.0 * 3600.0, 2, battery).unwrap();
        assert!(before.optimistic_gap() >= 1);
        model.lots.push(Lot {
            id: "resupply-sleeve".into(),
            resource: "sleeve-p1".into(),
            si: 1.0,
            quality: LotQuality::Accepted,
        });
        let after = joint_coverage(&model, "potable-water", 12.0 * 3600.0, 2, battery).unwrap();
        assert_eq!(after.joint_recoverable, after.independent_recoverable);
        assert_eq!(after.optimistic_gap(), 0);
    }

    #[test]
    fn joint_coverage_is_not_larger_than_independent_coverage() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let model = Model::load(&root.join("scenarios/base-06/model.toml")).expect("Base-06");
        let report = joint_coverage(
            &model,
            "potable-water",
            12.0 * 3600.0,
            2,
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        )
        .unwrap();
        assert!(report.joint_recoverable <= report.independent_recoverable);
        assert!(report.optimistic_gap() >= 1);
    }

    #[test]
    fn joint_shared_stock_breaks_an_independently_recoverable_pair() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let model = Model::load(&root.join("scenarios/base-06/model.toml")).expect("Base-06");
        let pair = vec!["pump-p1".to_string(), "pump-p2".to_string()];
        let envelope = failure_envelope(&model, "potable-water", 12.0 * 3600.0, 2).unwrap();
        assert!(envelope.cases.iter().any(|case| case.failed == pair && case.recoverable));
        let events = vec![
            FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p1".into() },
            FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p2".into() },
        ];
        let report = simulate_multi_service_longevity_with_limits(
            &model,
            &["potable-water"],
            30.0 * 86400.0,
            &events,
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        )
        .unwrap();
        assert_eq!(report.survived_s, 10.0 * 86400.0);
        assert!(report.log.iter().any(|entry| entry.blocked_by.is_some()));
    }

    #[test]
    fn double_failure_envelope_counts_pairs() {
        let envelope = failure_envelope(&base05(), "potable-water", 12.0 * 3600.0, 2).unwrap();
        assert_eq!(envelope.total(), 46);
        assert!(envelope.recoverable < envelope.total());
        assert!(envelope.cases.iter().any(|case| {
            case.failed.contains(&"filter-1".to_string())
                && case.failed.contains(&"bus-1".to_string())
                && !case.recoverable
        }));
    }

    #[test]
    fn high_site_load_makes_energy_the_limiting_factor() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let model = Model::load(&root.join("scenarios/base-05/model.toml")).expect("Base-05");
        let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
        let events = vec![FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p1".into() }];
        let idle = simulate_multi_service_longevity_with_load(
            &model,
            &["potable-water", "workshop-operability"],
            30.0 * 86400.0,
            &events,
            battery,
            0.0,
        )
        .unwrap();
        let loaded = simulate_multi_service_longevity_with_load(
            &model,
            &["potable-water", "workshop-operability"],
            30.0 * 86400.0,
            &events,
            battery,
            3.0,
        )
        .unwrap();
        assert!(loaded.survived_s < idle.survived_s);
        assert!(loaded.log.iter().any(|entry| entry.blocked_by.as_deref() == Some("energy_depleted")));
    }

    #[test]
    fn longer_failure_intervals_extend_survival_horizon() {
        let make = |interval: u32| -> Vec<FailureEvent> {
            (1..=(89 / interval))
                .map(|index| FailureEvent { at_s: f64::from(index * interval) * 86400.0, asset: "pump-p1".into() })
                .collect()
        };
        let battery = Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 });
        let short = simulate_multi_service_longevity_with_limits(
            &base05(),
            &["potable-water", "workshop-operability"],
            90.0 * 86400.0,
            &make(10),
            battery,
        )
        .unwrap();
        let long = simulate_multi_service_longevity_with_limits(
            &base05(),
            &["potable-water", "workshop-operability"],
            90.0 * 86400.0,
            &make(20),
            battery,
        )
        .unwrap();
        assert!(long.survived_s > short.survived_s);
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
    #[test]
    fn revocation_blocks_further_repairs() {
        let policy =
            RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 8.0 * 3600.0, approved_plan_digest: None };
        let events = vec![FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p1".into() }];
        let report = simulate_recovery_with_revocation(
            &base05(),
            "potable-water",
            30.0 * 86400.0,
            &events,
            policy,
            5.0 * 86400.0,
        )
        .unwrap();
        assert_eq!(report.survived_s, 10.0 * 86400.0);
        assert!(report.log.iter().any(|entry| entry.blocked_by.as_deref() == Some("approval_revoked")));
    }

    #[test]
    fn revocation_before_start_blocks_immediately() {
        let policy =
            RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 8.0 * 3600.0, approved_plan_digest: None };
        let report =
            simulate_recovery_with_revocation(&base05(), "potable-water", 30.0 * 86400.0, &[], policy, 0.0).unwrap();
        assert_eq!(report.survived_s, 0.0);
        assert!(report.log.iter().any(|entry| entry.blocked_by.as_deref() == Some("approval_revoked")));
    }

    #[test]
    fn multi_service_recovery_respects_revocation() {
        let policy =
            RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 8.0 * 3600.0, approved_plan_digest: None };
        let events = vec![FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p1".into() }];
        let report = simulate_multi_service_recovery_with_revocation(
            &base05(),
            &["potable-water", "workshop-operability"],
            30.0 * 86400.0,
            &events,
            policy,
            5.0 * 86400.0,
        )
        .unwrap();
        assert_eq!(report.survived_s, 10.0 * 86400.0);
        assert!(report.log.iter().any(|entry| entry.blocked_by.as_deref() == Some("approval_revoked")));
    }

    #[test]
    fn multi_service_recovery_repairs_shared_outage() {
        let policy =
            RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 8.0 * 3600.0, approved_plan_digest: None };
        let events = vec![FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p1".into() }];
        let report = simulate_multi_service_recovery_with_policy(
            &base05(),
            &["potable-water", "workshop-operability"],
            30.0 * 86400.0,
            &events,
            policy,
        )
        .unwrap();
        assert_eq!(report.survived_s, 30.0 * 86400.0);
        assert_eq!(report.services.len(), 2);
        assert!(report.log.iter().filter(|entry| entry.receipt.is_some()).count() >= 2);
    }

    #[test]
    fn multi_service_recovery_handles_two_events() {
        let policy =
            RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 8.0 * 3600.0, approved_plan_digest: None };
        let events = vec![
            FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p1".into() },
            FailureEvent { at_s: 20.0 * 86400.0, asset: "lathe-1".into() },
        ];
        let report = simulate_multi_service_recovery_with_policy(
            &base05(),
            &["potable-water", "workshop-operability"],
            30.0 * 86400.0,
            &events,
            policy,
        )
        .unwrap();
        assert_eq!(report.survived_s, 30.0 * 86400.0);
        assert_eq!(report.log.iter().filter(|entry| entry.receipt.is_some()).count(), 3);
    }

    #[test]
    fn multi_service_recovery_blocks_on_missing_input() {
        let mut model = base05();
        model.lots.retain(|lot| lot.id != "lot-sleeve-01" && lot.id != "lot-bronze");
        let policy =
            RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 8.0 * 3600.0, approved_plan_digest: None };
        let events = vec![FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p1".into() }];
        let report = simulate_multi_service_recovery_with_policy(
            &model,
            &["potable-water", "workshop-operability"],
            30.0 * 86400.0,
            &events,
            policy,
        )
        .unwrap();
        assert_eq!(report.survived_s, 10.0 * 86400.0);
        assert!(report
            .log
            .iter()
            .any(|entry| entry.blocked_by.as_deref().is_some_and(|value| value.contains("missing resource"))));
    }

    #[test]
    fn multi_service_recovery_stops_on_short_lease() {
        let policy =
            RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 3600.0, approved_plan_digest: None };
        let events = vec![FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p1".into() }];
        let report = simulate_multi_service_recovery_with_policy(
            &base05(),
            &["potable-water", "workshop-operability"],
            30.0 * 86400.0,
            &events,
            policy,
        )
        .unwrap();
        assert!(report.survived_s <= 10.0 * 86400.0);
        assert!(report
            .log
            .iter()
            .any(|entry| entry.blocked_by.as_deref().is_some_and(|value| value.contains("lease"))));
    }
    #[test]
    fn oversized_initial_energy_is_rejected() {
        let mut model = base05();
        if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "electric-energy") {
            lot.si = 500.0 * 3_600_000.0;
        }
        let result = simulate_multi_service_longevity_with_limits(
            &model,
            &["potable-water"],
            30.0 * 86400.0,
            &[],
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        );
        assert!(result.is_err());
    }

    #[test]
    fn fitted_initial_energy_is_accepted() {
        let result = simulate_multi_service_longevity_with_limits(
            &base05(),
            &["potable-water"],
            30.0 * 86400.0,
            &[],
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        );
        assert!(result.is_ok());
    }
    #[test]
    fn repeated_partial_yield_runs_extend_the_horizon() {
        use poie_ontology::model::{Flow, Process, Verification, YieldSpec};
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
        model.add_process(Process {
            id: "sharpen-insert".into(),
            inputs: vec![
                Flow { resource: "worn-insert".into(), si: 1.0 },
                Flow { resource: "electric-energy".into(), si: 0.3 * 3_600_000.0 },
            ],
            outputs: vec![Flow { resource: "cutting-insert".into(), si: 1.0 }],
            occupies: vec!["gauge-1".into()],
            requires_capabilities: Vec::new(),
            restores: Vec::new(),
            duration_s: Some(3600.0),
            verification: Verification::Modeled,
            asset_mass_change_kg: 0.0,
            mass_tolerance: 0.01,
            process_yield: Some(YieldSpec {
                output: "cutting-insert".into(),
                fraction: 0.8,
                reject: "worn-insert-reject".into(),
            }),
        });
        if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "cutting-insert") {
            lot.si = 2.0;
        }
        if let Some(lot) = model.lots.iter_mut().find(|lot| lot.resource == "bronze-bar") {
            lot.si = 3.0;
        }
        let mut events: Vec<FailureEvent> = (1..=8)
            .map(|index| FailureEvent { at_s: f64::from(index * 10) * 86400.0, asset: "pump-p1".into() })
            .collect();
        events.push(FailureEvent { at_s: 35.0 * 86400.0, asset: "lathe-1".into() });
        events.sort_by(|left, right| left.at_s.total_cmp(&right.at_s));
        let report = simulate_multi_service_longevity_with_limits(
            &model,
            &["potable-water", "workshop-operability"],
            90.0 * 86400.0,
            &events,
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        )
        .unwrap();
        assert_eq!(report.survived_s / 86400.0, 60.0);
    }
    #[test]
    fn revocation_inside_a_plan_cancels_it() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
        let policy =
            RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 8.0 * 3600.0, approved_plan_digest: None };
        let report = simulate_recovery_with_revocation(&model, "potable-water", 86_400.0, &[], policy, 1.0).unwrap();
        assert_eq!(report.survived_s, 0.0);
        assert!(report.log.iter().all(|entry| entry.receipt.is_none()));
        assert!(report.log.iter().any(|entry| entry
            .blocked_by
            .as_deref()
            .is_some_and(|reason| reason.starts_with("approval_revoked_mid_plan"))));
    }
    #[test]
    fn denial_at_an_event_stops_after_prior_repairs() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
        let events = vec![
            FailureEvent { at_s: 10.0 * 86_400.0, asset: "pump-p1".into() },
            FailureEvent { at_s: 20.0 * 86_400.0, asset: "pump-p1".into() },
        ];
        let policy =
            RecoveryPolicy { decision: ApprovalDecision::Approved, lease_s: 8.0 * 3600.0, approved_plan_digest: None };
        let report =
            simulate_recovery_with_denial(&model, "potable-water", 30.0 * 86_400.0, &events, policy, 10.0 * 86_400.0)
                .unwrap();
        assert_eq!(report.survived_s, 10.0 * 86_400.0);
        assert!(report.log.iter().any(|entry| entry.blocked_by.as_deref() == Some("approval_denied_at_event:pump-p1")));
    }
    #[test]
    fn event_driven_repair_advances_the_clock() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
        let events = vec![
            FailureEvent { at_s: 0.0, asset: "pump-p1".into() },
            FailureEvent { at_s: 3600.0, asset: "pump-p1".into() },
        ];
        let report = simulate_multi_service_longevity_event_driven(
            &model,
            &["potable-water", "workshop-operability"],
            86_400.0,
            &events,
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
            0.0,
        )
        .unwrap();
        let times: Vec<f64> =
            report.log.iter().filter(|entry| entry.failed.is_some()).map(|entry| entry.at_s / 3600.0).collect();
        assert_eq!(times, vec![3.0, 6.0]);
    }
    #[test]
    fn resupply_adds_to_inventory() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let model = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
        let resupplies = vec![Resupply { at_s: 10.0 * 86_400.0, resource: "cutting-insert".into(), si: 1.0 }];
        let report = simulate_multi_service_longevity_with_resupply(
            &model,
            &["potable-water", "workshop-operability"],
            20.0 * 86_400.0,
            &[],
            &resupplies,
            Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
            0.0,
        )
        .unwrap();
        assert!(report.final_inventory["cutting-insert"] >= 5.0);
    }
}
