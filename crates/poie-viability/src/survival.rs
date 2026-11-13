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

use poie_ontology::model::{AssetStatus, Model};

use crate::closure::{analyze, BlockReason, ClosureOutcome};

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
}
