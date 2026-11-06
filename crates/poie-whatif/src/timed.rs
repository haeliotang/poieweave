//! Phase C2a: independent timed/storage What-if DTOs.
//!
//! C2a deliberately does not extend C1's snapshot/variant/result types: C1
//! digests and persisted rows stay byte-compatible. Timed runs use the
//! sandbox-loaded `TimedScenario` and never write live/SAP/federation state.

use std::collections::BTreeMap;

use poie_digest::sha256_of;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const TIMED_SNAPSHOT_SCHEMA: &str = "poie.timed-snapshot/0.1";
pub const TIMED_VARIANT_SCHEMA: &str = "poie.timed-variant/0.1";
pub const TIMED_RESULT_SCHEMA: &str = "poie.timed-result/0.1";
pub const TIMED_COMPARE_SCHEMA: &str = "poie.timed-compare/0.1";
pub const TIMED_CAVEAT: &str = "relative to this timed scenario and bounded model; not a physical feasibility proof";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TimedSnapshotInput {
    pub label: String,
    pub model_sha256: String,
    pub scenario_path: String,
    pub scenario_sha256: String,
    pub goal: String,
    pub min_verification: String,
    #[serde(default)]
    pub start_s: f64,
    pub horizon_s: f64,
    #[serde(default)]
    pub deadline_s: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimedSnapshot {
    pub schema: String,
    pub schema_version: String,
    pub id: String,
    pub label: String,
    pub snapshot_sha256: String,
    pub model_sha256: String,
    pub scenario_path: String,
    pub scenario_sha256: String,
    pub goal: String,
    pub min_verification: String,
    pub start_s: f64,
    pub horizon_s: f64,
    pub now_s: f64,
    pub deadline_s: Option<f64>,
    pub created_at: i64,
    pub created_by: String,
}

impl TimedSnapshot {
    pub fn create(input: TimedSnapshotInput, id: String, now_ms: i64, created_by: String) -> Result<Self, String> {
        if !matches!(input.goal.split_once(':'), Some(("service", v)) if !v.is_empty())
            && !matches!(input.goal.split_once(':'), Some(("restore", v)) if !v.is_empty())
        {
            return Err("invalid_parameter: goal must be service:<id> or restore:<asset>".into());
        }
        if input.model_sha256.len() != 64 || !input.model_sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("model_mismatch: modelSha256 must be a 64-hex digest".into());
        }
        if input.scenario_sha256.len() != 64 || !input.scenario_sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("scenario_mismatch: scenarioSha256 must be a 64-hex digest".into());
        }
        if input.scenario_path.trim().is_empty() {
            return Err("invalid_parameter: scenarioPath is required".into());
        }
        if !input.start_s.is_finite()
            || !input.horizon_s.is_finite()
            || input.horizon_s <= 0.0
            || input.start_s < 0.0
            || input.start_s > input.horizon_s
        {
            return Err("invalid_parameter: startS must be inside [0, horizonS]".into());
        }
        if let Some(deadline) = input.deadline_s {
            if !deadline.is_finite() || deadline <= input.start_s || deadline > input.horizon_s {
                return Err("invalid_parameter: deadlineS must be inside (startS, horizonS]".into());
            }
        }
        let mut snapshot = TimedSnapshot {
            schema: TIMED_SNAPSHOT_SCHEMA.into(),
            schema_version: "0.1".into(),
            id,
            label: input.label,
            snapshot_sha256: String::new(),
            model_sha256: input.model_sha256.to_ascii_lowercase(),
            scenario_path: input.scenario_path,
            scenario_sha256: input.scenario_sha256.to_ascii_lowercase(),
            goal: input.goal,
            min_verification: input.min_verification,
            start_s: input.start_s,
            horizon_s: input.horizon_s,
            now_s: 0.0,
            deadline_s: input.deadline_s,
            created_at: now_ms,
            created_by,
        };
        snapshot.snapshot_sha256 = timed_snapshot_digest(&snapshot)?;
        Ok(snapshot)
    }
}

pub fn timed_snapshot_digest(snapshot: &TimedSnapshot) -> Result<String, String> {
    let mut value = serde_json::to_value(snapshot).map_err(|error| error.to_string())?;
    if let Some(object) = value.as_object_mut() {
        for key in ["snapshotSha256", "scenarioPath", "id", "createdAt", "createdBy"] {
            object.remove(key);
        }
    }
    sha256_of(TIMED_SNAPSHOT_SCHEMA, &value).map_err(|error| error.to_string())
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TimedVariantInput {
    pub snapshot_id: String,
    pub label: String,
    #[serde(default)]
    pub description: String,
    pub start_mode: String,
    #[serde(default)]
    pub start_s: Option<f64>,
    #[serde(default)]
    pub deadline_s: Option<f64>,
    #[serde(default)]
    pub storage_discharge_w: BTreeMap<String, f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimedVariant {
    pub schema: String,
    pub schema_version: String,
    pub id: String,
    pub snapshot_id: String,
    pub label: String,
    pub description: String,
    pub start_mode: String,
    pub start_s: Option<f64>,
    pub deadline_s: Option<f64>,
    pub storage_discharge_w: BTreeMap<String, f64>,
    pub created_at: i64,
    pub created_by: String,
}

impl TimedVariant {
    pub fn create(input: TimedVariantInput, id: String, now_ms: i64, created_by: String) -> Result<Self, String> {
        if input.start_mode != "beginning" && input.start_mode != "at" {
            return Err("invalid_parameter: startMode must be beginning or at".into());
        }
        if input.start_mode == "at" {
            let Some(start) = input.start_s else {
                return Err("invalid_parameter: startMode=at requires startS".into());
            };
            if !start.is_finite() || start < 0.0 {
                return Err("invalid_parameter: startS must be non-negative and finite".into());
            }
        } else if input.start_s.is_some_and(|value| value != 0.0) {
            return Err("invalid_parameter: startMode=beginning requires startS=0/null".into());
        }
        for (resource, watts) in &input.storage_discharge_w {
            if resource.trim().is_empty() || !watts.is_finite() || *watts < 0.0 {
                return Err("invalid_parameter: storageDischargeW must be non-negative finite watts".into());
            }
        }
        Ok(TimedVariant {
            schema: TIMED_VARIANT_SCHEMA.into(),
            schema_version: "0.1".into(),
            id,
            snapshot_id: input.snapshot_id,
            label: input.label,
            description: input.description,
            start_mode: input.start_mode,
            start_s: input.start_s,
            deadline_s: input.deadline_s,
            storage_discharge_w: input.storage_discharge_w,
            created_at: now_ms,
            created_by,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimedSettings {
    pub start_mode: String,
    pub start_s: f64,
    pub horizon_s: f64,
    pub deadline_s: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimedOutcomeKind {
    pub timed: String,
    pub caveat: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimedResult {
    pub schema: String,
    pub schema_version: String,
    pub result_id: String,
    pub variant_id: String,
    pub snapshot_id: String,
    pub run_id: String,
    pub mode: String,
    pub settings: TimedSettings,
    pub outcome: TimedOutcomeKind,
    pub completion_s: Option<f64>,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub residuals: Option<Value>,
    #[serde(default)]
    pub abstained: bool,
    #[serde(default)]
    pub abstain_reason: Option<String>,
    pub result_sha256: String,
    pub created_at: i64,
}

impl TimedResult {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        snapshot: &TimedSnapshot,
        variant: &TimedVariant,
        result_id: String,
        run_id: String,
        start_s: f64,
        deadline_s: Option<f64>,
        verdict: &str,
        completion_s: Option<f64>,
        warnings: Vec<String>,
        now_ms: i64,
    ) -> Result<Self, String> {
        let abstained = matches!(verdict, "unknown" | "search_limit");
        let abstain_reason = match verdict {
            "unknown" => Some("unknown_state".to_string()),
            "search_limit" => Some("search_limit".to_string()),
            _ => None,
        };
        let residuals = json!({
            "completionS": completion_s,
            "horizonS": snapshot.horizon_s,
            "warningCount": warnings.len(),
            "verdict": verdict,
        });
        let mut result = TimedResult {
            schema: TIMED_RESULT_SCHEMA.into(),
            schema_version: "0.1".into(),
            result_id,
            variant_id: variant.id.clone(),
            snapshot_id: snapshot.id.clone(),
            run_id,
            mode: "timed".into(),
            settings: TimedSettings {
                start_mode: variant.start_mode.clone(),
                start_s,
                horizon_s: snapshot.horizon_s,
                deadline_s,
            },
            outcome: TimedOutcomeKind { timed: verdict.into(), caveat: TIMED_CAVEAT.into() },
            completion_s,
            warnings,
            residuals: Some(residuals),
            abstained,
            abstain_reason,
            result_sha256: String::new(),
            created_at: now_ms,
        };
        result.result_sha256 = timed_result_digest(snapshot, variant, &result)?;
        Ok(result)
    }
}

pub fn timed_result_digest(
    snapshot: &TimedSnapshot,
    variant: &TimedVariant,
    result: &TimedResult,
) -> Result<String, String> {
    let mut value = serde_json::to_value(result).map_err(|error| error.to_string())?;
    if let Some(object) = value.as_object_mut() {
        object.remove("resultSha256");
        object.remove("resultId");
        object.remove("createdAt");
        object.remove("runId");
    }
    let payload = json!({
        "snapshotSha256": snapshot.snapshot_sha256,
        "scenarioSha256": snapshot.scenario_sha256,
        "modelSha256": snapshot.model_sha256,
        "variant": {
            "startMode": variant.start_mode,
            "startS": variant.start_s,
            "deadlineS": variant.deadline_s,
            "storageDischargeW": variant.storage_discharge_w,
        },
        "settings": value.get("settings").cloned().unwrap_or(Value::Null),
        "outcome": value.get("outcome").cloned().unwrap_or(Value::Null),
        "completionS": value.get("completionS").cloned().unwrap_or(Value::Null),
        "warnings": value.get("warnings").cloned().unwrap_or(Value::Null),
        "residuals": value.get("residuals").cloned().unwrap_or(Value::Null),
        "abstained": value.get("abstained").cloned().unwrap_or(Value::Bool(false)),
        "abstainReason": value.get("abstainReason").cloned().unwrap_or(Value::Null),
    });
    sha256_of("poie.timed-result/0.1", &payload).map_err(|error| error.to_string())
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimedCompareRow {
    pub result_id: String,
    pub variant_id: String,
    pub outcome: TimedOutcomeKind,
    pub completion_s: Option<f64>,
    pub delta: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimedCompareResult {
    pub schema: String,
    pub schema_version: String,
    pub snapshot_id: String,
    pub baseline_result_id: Option<String>,
    pub rows: Vec<TimedCompareRow>,
    pub sort_key: String,
    pub complete: bool,
    pub warnings: Vec<String>,
}

const VERDICT_ORDER: [&str; 8] = [
    "feasible",
    "blocked",
    "unknown",
    "search_limit",
    "too_late",
    "insufficient_resource",
    "capacity_conflict",
    "power_limited",
];

fn verdict_rank(value: &str) -> usize {
    VERDICT_ORDER.iter().position(|entry| *entry == value).unwrap_or(VERDICT_ORDER.len())
}

/// Deterministic compare for timed rows from one snapshot. A missing baseline
/// leaves every delta null; a null completion on either side also yields null.
pub fn compare_timed(
    snapshot_id: &str,
    baseline_result_id: Option<&str>,
    mut results: Vec<TimedResult>,
) -> Result<TimedCompareResult, String> {
    let mut warnings = Vec::new();
    results.sort_by(|left, right| left.result_id.cmp(&right.result_id));
    results.dedup_by(|left, right| left.result_id == right.result_id);
    if results.iter().any(|result| result.snapshot_id != snapshot_id) {
        return Err("invalid_parameter: timed compare requires one snapshot".into());
    }
    let baseline = baseline_result_id.and_then(|id| results.iter().find(|result| result.result_id == id)).cloned();
    if baseline_result_id.is_some() && baseline.is_none() {
        warnings.push("baseline_not_in_results".into());
    }
    results.sort_by(|left, right| {
        verdict_rank(&left.outcome.timed)
            .cmp(&verdict_rank(&right.outcome.timed))
            .then_with(|| match (left.completion_s, right.completion_s) {
                (Some(a), Some(b)) => a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            })
            .then_with(|| left.result_id.cmp(&right.result_id))
    });
    let rows = results
        .into_iter()
        .map(|result| {
            let delta = match (&baseline, result.completion_s) {
                (Some(baseline), Some(completion)) => match baseline.completion_s {
                    Some(base_completion) => json!({"completionDeltaS": completion - base_completion}),
                    None => json!({"completionDeltaS": null}),
                },
                _ => json!({"completionDeltaS": null}),
            };
            TimedCompareRow {
                result_id: result.result_id,
                variant_id: result.variant_id,
                outcome: result.outcome,
                completion_s: result.completion_s,
                delta,
            }
        })
        .collect();
    Ok(TimedCompareResult {
        schema: TIMED_COMPARE_SCHEMA.into(),
        schema_version: "0.1".into(),
        snapshot_id: snapshot_id.into(),
        baseline_result_id: baseline.as_ref().map(|result| result.result_id.clone()),
        rows,
        sort_key: "timedVerdict|completionS|resultId".into(),
        complete: true,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> TimedSnapshot {
        TimedSnapshot::create(
            TimedSnapshotInput {
                label: "base-01-timed".into(),
                model_sha256: "a".repeat(64),
                scenario_path: "base-01-timed/scenario.toml".into(),
                scenario_sha256: "5cbfc5a5711b316318fd244c89149b9ef58b6f5d8c417c4747638dc7c1c205f3".into(),
                goal: "restore:pump-p1".into(),
                min_verification: "demonstrated".into(),
                start_s: 0.0,
                horizon_s: 86_400.0,
                deadline_s: None,
            },
            "snap-1".into(),
            1,
            "test".into(),
        )
        .unwrap()
    }

    #[test]
    fn timed_snapshot_digest_matches_design_vector() {
        let snapshot = snapshot();

        assert_eq!(snapshot.now_s, 0.0);
        assert_eq!(snapshot.snapshot_sha256, "e694e44a32754e876008a99fef02a59904d126f3488a4156519445b27218ef65");
    }

    #[test]
    fn timed_compare_nulls_delta_when_completion_is_missing() {
        let snapshot = snapshot();
        let variant = TimedVariant::create(
            TimedVariantInput {
                snapshot_id: snapshot.id.clone(),
                label: "storage".into(),
                description: String::new(),
                start_mode: "beginning".into(),
                start_s: None,
                deadline_s: None,
                storage_discharge_w: [("electric-energy".into(), 3000.0)].into_iter().collect(),
            },
            "var-1".into(),
            1,
            "test".into(),
        )
        .unwrap();
        let blocked = TimedResult::create(
            &snapshot,
            &variant,
            "res-1".into(),
            "run-1".into(),
            0.0,
            None,
            "power_limited",
            None,
            vec![],
            1,
        )
        .unwrap();
        let feasible = TimedResult::create(
            &snapshot,
            &variant,
            "res-2".into(),
            "run-2".into(),
            0.0,
            None,
            "feasible",
            Some(10.0),
            vec![],
            1,
        )
        .unwrap();
        let compared = compare_timed(&snapshot.id, None, vec![blocked, feasible]).unwrap();
        assert!(compared.rows.iter().all(|row| row.delta["completionDeltaS"].is_null()));
        assert_eq!(compared.rows[0].outcome.timed, "feasible");
    }

    #[test]
    fn timed_result_abstains_for_unknown_verdict() {
        let snapshot = TimedSnapshot::create(
            TimedSnapshotInput {
                label: "base-01-timed".into(),
                model_sha256: "a".repeat(64),
                scenario_path: "base-01-timed/scenario.toml".into(),
                scenario_sha256: "5cbfc5a5711b316318fd244c89149b9ef58b6f5d8c417c4747638dc7c1c205f3".into(),
                goal: "restore:pump-p1".into(),
                min_verification: "demonstrated".into(),
                start_s: 0.0,
                horizon_s: 86_400.0,
                deadline_s: None,
            },
            "snap".into(),
            1,
            "test".into(),
        )
        .unwrap();
        let variant = TimedVariant::create(
            TimedVariantInput {
                snapshot_id: snapshot.id.clone(),
                label: "v".into(),
                description: String::new(),
                start_mode: "beginning".into(),
                start_s: None,
                deadline_s: None,
                storage_discharge_w: Default::default(),
            },
            "var".into(),
            1,
            "test".into(),
        )
        .unwrap();
        let result =
            TimedResult::create(&snapshot, &variant, "res".into(), "run".into(), 0.0, None, "unknown", None, vec![], 1)
                .unwrap();
        assert!(result.abstained);
        assert_eq!(result.abstain_reason.as_deref(), Some("unknown_state"));
        assert!(result.residuals.is_some());
    }
}
