// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 self-sustaining metrics: Survival Horizon, Recovery Coverage and
//! Regenerative Capacity. These are model-level measures, not physical
//! forecasts.

use crate::survival::{FailureEnvelope, SurvivalReport};

#[derive(Clone, Debug, PartialEq)]
pub struct SurvivalHorizonMetric {
    pub service: String,
    pub seconds: f64,
}

impl SurvivalHorizonMetric {
    pub fn from_report(report: &SurvivalReport) -> Self {
        Self { service: report.service.clone(), seconds: report.survived_s }
    }

    pub fn days(&self) -> f64 {
        self.seconds / 86400.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecoveryCoverageMetric {
    pub recoverable: usize,
    pub total: usize,
}

impl RecoveryCoverageMetric {
    pub fn from_envelope(envelope: &FailureEnvelope) -> Self {
        Self { recoverable: envelope.recoverable, total: envelope.total() }
    }

    pub fn ratio(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.recoverable as f64 / self.total as f64
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RegenerativeCapacityMetric {
    pub baseline_seconds: f64,
    pub regenerated_seconds: f64,
}

impl RegenerativeCapacityMetric {
    pub fn between(baseline: &SurvivalReport, regenerated: &SurvivalReport) -> Self {
        Self { baseline_seconds: baseline.survived_s, regenerated_seconds: regenerated.survived_s }
    }

    pub fn delta_seconds(&self) -> f64 {
        self.regenerated_seconds - self.baseline_seconds
    }

    pub fn delta_days(&self) -> f64 {
        self.delta_seconds() / 86400.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RechargeBoundary {
    pub average_recharge_kw: f64,
    pub threshold_load_kw: f64,
}

impl RechargeBoundary {
    pub fn margin_kw(&self) -> f64 {
        self.threshold_load_kw - self.average_recharge_kw
    }

    pub fn load_is_sustainable(&self, load_kw: f64) -> bool {
        load_kw <= self.average_recharge_kw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizon_converts_to_days() {
        let horizon = SurvivalHorizonMetric { service: "potable-water".into(), seconds: 40.0 * 86400.0 };
        assert!((horizon.days() - 40.0).abs() < 1e-9);
    }

    #[test]
    fn coverage_ratio_handles_empty_envelope() {
        let coverage = RecoveryCoverageMetric { recoverable: 0, total: 0 };
        assert_eq!(coverage.ratio(), 0.0);
        let coverage = RecoveryCoverageMetric { recoverable: 11, total: 46 };
        assert!((coverage.ratio() - 11.0 / 46.0).abs() < 1e-12);
    }

    #[test]
    fn regenerative_capacity_reports_delta() {
        let capacity =
            RegenerativeCapacityMetric { baseline_seconds: 40.0 * 86400.0, regenerated_seconds: 60.0 * 86400.0 };
        assert!((capacity.delta_days() - 20.0).abs() < 1e-9);
    }
    #[test]
    fn recharge_boundary_reports_margin_and_sustainability() {
        let boundary = RechargeBoundary { average_recharge_kw: 2.5, threshold_load_kw: 2.6 };
        assert!((boundary.margin_kw() - 0.1).abs() < 1e-9);
        assert!(boundary.load_is_sustainable(2.5));
        assert!(!boundary.load_is_sustainable(2.6));
    }
}
