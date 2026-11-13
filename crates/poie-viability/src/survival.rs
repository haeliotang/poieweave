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
        add_recharge(&current, &mut inventory, at - now, battery);
        now = at;
        if let Some(asset) = current.assets.get_mut(&event.asset) {
            asset.status = AssetStatus::Failed;
        } else {
            return Err(EnvelopeError(format!("unknown failed asset {}", event.asset)));
        }
        match restore_services(&mut current, &services, now, horizon_s, &mut inventory) {
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
                    service: service_label.clone(),
                    horizon_s,
                    survived_s: now,
                    log,
                    final_inventory: inventory,
                });
            }
        }
    }
    add_recharge(&current, &mut inventory, horizon_s - now, battery);
    Ok(SurvivalReport {
        service: service_label.clone(),
        horizon_s,
        survived_s: horizon_s,
        log,
        final_inventory: inventory,
    })
}

fn restore_services(
    model: &mut Model,
    services: &[&Service],
    now_s: f64,
    horizon_s: f64,
    inventory: &mut BTreeMap<String, f64>,
) -> Result<Vec<String>, String> {
    let mut repaired = Vec::new();
    for service in services {
        repaired.extend(restore_service(model, service, now_s, horizon_s, inventory)?);
    }
    Ok(repaired)
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

fn recharge_j_per_s(model: &Model) -> Option<f64> {
    let process = model.processes.iter().find(|process| process.id == "recharge-energy")?;
    let duration = process.duration_s?;
    let output = process.outputs.iter().find(|output| output.resource == "electric-energy")?;
    if duration <= 0.0 {
        return None;
    }
    Some(output.si / duration)
}

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
}
