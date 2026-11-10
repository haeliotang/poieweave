// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Phase C1: static, deterministic what-if snapshots and variants.
//!
//! C1 only rewrites an in-memory `Model` copy and runs graph/plan. It never
//! touches live state, SAP, federation, approvals or dispatch.

use std::collections::BTreeMap;

use poie_digest::sha256_of;
use poie_ontology::graph::{self, Goal, Query, Verdict};
use poie_ontology::model::{Model, Patch, Verification};
use poie_plan::plan::{plan, Plan, PlanError, PlanningGoal, PlanningRequest};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub mod promotion;
pub mod replay;
pub mod timed;
pub use timed::*;

pub const SNAPSHOT_SCHEMA: &str = "poie.whatif-snapshot/0.1";
pub const VARIANT_SCHEMA: &str = "poie.whatif-variant/0.1";
pub const RESULT_SCHEMA: &str = "poie.whatif-result/0.1";
pub const COMPARE_SCHEMA: &str = "poie.whatif-compare/0.1";
pub const DEFAULT_MAX_DEPTH: usize = 12;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SnapshotInput {
    pub label: String,
    pub model_sha256: String,
    #[serde(default)]
    pub evidence_sha256: Option<String>,
    pub goal: String,
    pub min_verification: String,
    pub horizon_s: f64,
    pub now_s: f64,
    pub start_s: f64,
    #[serde(default)]
    pub deadline_s: Option<f64>,
    pub max_depth: usize,
    pub max_actions: usize,
    pub seed: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub schema: String,
    pub schema_version: String,
    pub id: String,
    pub label: String,
    pub snapshot_sha256: String,
    pub model_sha256: String,
    pub evidence_sha256: Option<String>,
    pub goal: String,
    pub min_verification: String,
    pub horizon_s: f64,
    pub now_s: f64,
    pub start_s: f64,
    pub deadline_s: Option<f64>,
    pub max_depth: usize,
    pub max_actions: usize,
    pub seed: u64,
    pub created_at: i64,
    pub created_by: String,
}

impl Snapshot {
    pub fn create(input: SnapshotInput, id: String, now_ms: i64, created_by: String) -> Result<Snapshot, String> {
        if !matches!(input.goal.split_once(':'), Some(("service", value)) if !value.is_empty())
            && !matches!(input.goal.split_once(':'), Some(("restore", value)) if !value.is_empty())
        {
            return Err("invalid_patch: goal must be service:<id> or restore:<asset>".into());
        }
        if input.horizon_s <= input.now_s {
            return Err("invalid_parameter: horizonS must exceed nowS".into());
        }
        if input.max_actions == 0 || input.max_depth == 0 {
            return Err("invalid_parameter: maxActions/maxDepth must be positive".into());
        }
        let mut snapshot = Snapshot {
            schema: SNAPSHOT_SCHEMA.into(),
            schema_version: "0.1".into(),
            id,
            label: input.label,
            snapshot_sha256: String::new(),
            model_sha256: input.model_sha256,
            evidence_sha256: input.evidence_sha256,
            goal: input.goal,
            min_verification: input.min_verification,
            horizon_s: input.horizon_s,
            now_s: input.now_s,
            start_s: input.start_s,
            deadline_s: input.deadline_s,
            max_depth: input.max_depth,
            max_actions: input.max_actions,
            seed: input.seed,
            created_at: now_ms,
            created_by,
        };
        snapshot.snapshot_sha256 = snapshot_digest(&snapshot)?;
        Ok(snapshot)
    }
    pub fn verify_model(&self, model: &Model) -> Result<(), String> {
        let actual = poie_objects_model_digest(model)?;
        if actual != self.model_sha256 {
            return Err(format!("model_mismatch: snapshot {} expected {}", self.id, self.model_sha256));
        }
        Ok(())
    }
}

// Keep `poie-whatif` independent of `poie-objects`' runtime surface while using
// the same canonical model digest domain.
fn poie_objects_model_digest(model: &Model) -> Result<String, String> {
    poie_objects::model_digest(model).map_err(|error| error.to_string())
}

pub fn snapshot_digest(snapshot: &Snapshot) -> Result<String, String> {
    let mut value = serde_json::to_value(snapshot).map_err(|error| error.to_string())?;
    if let Some(object) = value.as_object_mut() {
        for key in ["snapshotSha256", "id", "createdAt", "createdBy", "seed"] {
            object.remove(key);
        }
    }
    sha256_of("poie.snapshot/0.1", &value).map_err(|error| error.to_string())
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PatchDto {
    #[serde(default)]
    pub asset_status: BTreeMap<String, String>,
    #[serde(default)]
    pub lot_quantity: BTreeMap<String, String>,
    #[serde(default)]
    pub lot_quality: BTreeMap<String, String>,
    #[serde(default)]
    pub lot_resource: BTreeMap<String, String>,
    #[serde(default)]
    pub process_verification: BTreeMap<String, String>,
    #[serde(default)]
    pub asset_requires: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub asset_requires_any: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub add_asset: Vec<Value>,
    #[serde(default)]
    pub asset_degraded: BTreeMap<String, Value>,
    #[serde(default)]
    pub asset_capability_verification: BTreeMap<String, BTreeMap<String, String>>,
    #[serde(default)]
    pub add_resource: Vec<Value>,
    #[serde(default)]
    pub add_lot: Vec<Value>,
    #[serde(default)]
    pub add_process: Vec<Value>,
}

impl PatchDto {
    pub fn to_patch(&self) -> Result<Patch, String> {
        let enum_value =
            |value: &str| serde_json::from_value::<serde_json::Value>(json!(value)).map_err(|error| error.to_string());
        let mut patch = Patch::default();
        for (id, status) in &self.asset_status {
            let _ = enum_value(status)?;
            patch.asset_status.insert(
                id.clone(),
                serde_json::from_value(json!(status))
                    .map_err(|error| format!("invalid_patch: assetStatus {id}: {error}"))?,
            );
        }
        patch.lot_quantity = self.lot_quantity.clone();
        for (id, quality) in &self.lot_quality {
            patch.lot_quality.insert(
                id.clone(),
                serde_json::from_value(json!(quality))
                    .map_err(|error| format!("invalid_patch: lotQuality {id}: {error}"))?,
            );
        }
        patch.lot_resource = self.lot_resource.clone();
        for (id, level) in &self.process_verification {
            patch.process_verification.insert(
                id.clone(),
                serde_json::from_value(json!(level))
                    .map_err(|error| format!("invalid_patch: processVerification {id}: {error}"))?,
            );
        }
        patch.asset_requires = self.asset_requires.clone();
        patch.asset_requires_any = self.asset_requires_any.clone();
        patch.add_asset = serde_json::from_value(Value::Array(self.add_asset.clone()))
            .map_err(|error| format!("invalid_patch: addAsset: {error}"))?;
        for (id, policy) in &self.asset_degraded {
            patch.asset_degraded.insert(
                id.clone(),
                serde_json::from_value(policy.clone())
                    .map_err(|error| format!("invalid_patch: assetDegraded {id}: {error}"))?,
            );
        }
        for (id, capabilities) in &self.asset_capability_verification {
            let mut converted = BTreeMap::new();
            for (capability, level) in capabilities {
                converted.insert(
                    capability.clone(),
                    serde_json::from_value(json!(level))
                        .map_err(|error| format!("invalid_patch: capabilityVerification {id}.{capability}: {error}"))?,
                );
            }
            patch.asset_capability_verification.insert(id.clone(), converted);
        }
        patch.add_resource = serde_json::from_value(Value::Array(self.add_resource.clone()))
            .map_err(|error| format!("invalid_patch: addResource: {error}"))?;
        patch.add_lot = serde_json::from_value(Value::Array(self.add_lot.clone()))
            .map_err(|error| format!("invalid_patch: addLot: {error}"))?;
        patch.add_process = serde_json::from_value(Value::Array(self.add_process.clone()))
            .map_err(|error| format!("invalid_patch: addProcess: {error}"))?;
        Ok(patch)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VariantInput {
    pub snapshot_id: String,
    pub label: String,
    #[serde(default)]
    pub description: String,
    pub patch: PatchDto,
    #[serde(default)]
    pub goal: Option<String>,
    #[serde(default)]
    pub min_verification: Option<String>,
    #[serde(default)]
    pub deadline_s: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variant {
    pub schema: String,
    pub schema_version: String,
    pub id: String,
    pub snapshot_id: String,
    pub label: String,
    pub description: String,
    pub patch: PatchDto,
    pub goal: Option<String>,
    pub min_verification: Option<String>,
    pub deadline_s: Option<f64>,
    pub created_at: i64,
    pub created_by: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub now_s: f64,
    pub start_s: f64,
    pub horizon_s: f64,
    pub max_depth: usize,
    pub max_actions: usize,
    pub deadline_s: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutcomeKind {
    pub structure: Option<String>,
    pub plan: Option<String>,
    pub caveat: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionView {
    pub action_id: String,
    pub process: String,
    pub target: String,
    pub start_s: f64,
    pub end_s: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CauseView {
    pub kind: String,
    pub subject: String,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyView {
    pub resource: String,
    pub max_si: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariantResult {
    pub schema: String,
    pub schema_version: String,
    pub result_id: String,
    pub variant_id: String,
    pub snapshot_id: String,
    pub run_id: String,
    pub mode: String,
    pub settings: Settings,
    pub outcome: OutcomeKind,
    pub plan_fingerprint: Option<String>,
    pub plan_sha256: Option<String>,
    pub completion_s: Option<f64>,
    pub actions: Vec<ActionView>,
    pub root_causes: Vec<CauseView>,
    pub external_dependencies: Vec<DependencyView>,
    pub result_sha256: String,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub residuals: Option<Value>,
    #[serde(default)]
    pub abstained: bool,
    #[serde(default)]
    pub abstain_reason: Option<String>,
    #[serde(default)]
    pub counterevidence: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompareRow {
    pub result_id: String,
    pub variant_id: String,
    pub mode: String,
    pub outcome: Value,
    pub completion_s: Option<f64>,
    pub plan_fingerprint: Option<String>,
    pub delta: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompareResult {
    pub schema: String,
    pub schema_version: String,
    pub snapshot_id: String,
    pub baseline_result_id: Option<String>,
    pub rows: Vec<CompareRow>,
    pub sort_key: String,
    pub complete: bool,
    pub warnings: Vec<String>,
}

fn parse_goal(value: &str) -> Result<Goal, String> {
    match value.split_once(':') {
        Some(("service", id)) if !id.is_empty() => Ok(Goal::Service(id.into())),
        Some(("restore", id)) if !id.is_empty() => Ok(Goal::Restore(id.into())),
        _ => Err(format!("invalid_patch: goal must be service:<id> or restore:<asset>, got {value:?}")),
    }
}

fn parse_verification(value: &str) -> Result<Verification, String> {
    serde_json::from_value(json!(value)).map_err(|error| format!("invalid_parameter: minVerification: {error}"))
}

pub fn run_structure(model: &Model, snapshot: &Snapshot, variant: &Variant) -> Result<VariantResult, String> {
    snapshot.verify_model(model)?;
    let patch = variant.patch.to_patch()?;
    let patched = model.patched(&patch).map_err(|error| format!("invalid_patch: {error}"))?;
    let goal = parse_goal(variant.goal.as_deref().unwrap_or(&snapshot.goal))?;
    let min_verification =
        parse_verification(variant.min_verification.as_deref().unwrap_or(&snapshot.min_verification))?;
    let result = graph::solve(&patched, &Query { goal, min_verification, max_depth: snapshot.max_depth });
    let structure = match result.verdict {
        Verdict::Candidate => "candidate",
        Verdict::Blocked => "blocked",
        Verdict::Unknown => "unknown",
        Verdict::SearchLimit => "search_limit",
    };
    let causes = result
        .blockers
        .iter()
        .map(|blocker| CauseView {
            kind: "blocker".into(),
            subject: blocker.subject.clone(),
            detail: blocker.detail.clone(),
        })
        .collect();
    let dependencies = result
        .steps
        .iter()
        .flat_map(|step| {
            step.allocations
                .iter()
                .map(|allocation| DependencyView { resource: allocation.resource.clone(), max_si: allocation.si })
        })
        .collect();
    let actions = result
        .steps
        .iter()
        .map(|step| ActionView {
            action_id: step.process.clone(),
            process: step.process.clone(),
            target: step.providers.first().map(|(_, provider, _)| provider.clone()).unwrap_or_default(),
            start_s: 0.0,
            end_s: 0.0,
        })
        .collect();
    let mut result =
        finish_result(snapshot, variant, "structure", structure, None, None, causes, dependencies, actions)?;
    result.residuals = Some(json!({
        "rootCauses": result.root_causes.len(),
        "actions": result.actions.len(),
        "externalDependencies": result.external_dependencies.len(),
    }));
    result.result_sha256 = result_digest(snapshot, variant, &result)?;
    Ok(result)
}

pub fn run_plan(model: &Model, snapshot: &Snapshot, variant: &Variant) -> Result<VariantResult, String> {
    snapshot.verify_model(model)?;
    let patch = variant.patch.to_patch()?;
    let patched = model.patched(&patch).map_err(|error| format!("invalid_patch: {error}"))?;
    let goal = variant.goal.clone().unwrap_or_else(|| snapshot.goal.clone());
    let min_verification =
        parse_verification(variant.min_verification.as_deref().unwrap_or(&snapshot.min_verification))?;
    let deadline = variant.deadline_s.or(snapshot.deadline_s);
    let request = PlanningRequest {
        model: &patched,
        now_s: snapshot.now_s,
        horizon_s: snapshot.horizon_s,
        goals: vec![PlanningGoal { goal: parse_goal(&goal)?, deadline_s: deadline, priority: 10 }],
        min_verification,
        max_actions: snapshot.max_actions,
    };
    let plan_result = match plan(&request) {
        Ok(value) => value,
        Err(PlanError::InvalidRequest(detail)) => return Err(format!("invalid_parameter: {detail}")),
        Err(PlanError::Model(detail) | PlanError::Schedule(detail)) => return Err(format!("unknown: {detail}")),
    };
    let (outcome, warnings) = plan_outcome(&plan_result);
    let actions = plan_result
        .actions
        .iter()
        .map(|action| ActionView {
            action_id: action.action_id.clone(),
            process: action.process.clone(),
            target: action.target.clone(),
            start_s: action.start_s,
            end_s: action.end_s,
        })
        .collect();
    let causes = plan_result
        .goals_unmet
        .iter()
        .map(|failure| CauseView {
            kind: "goal_unmet".into(),
            subject: failure.goal.clone(),
            detail: failure.reason.clone(),
        })
        .collect();
    let dependencies = plan_result
        .actions
        .iter()
        .flat_map(|action| {
            action.allocations.iter().map(|(resource, si)| DependencyView { resource: resource.clone(), max_si: *si })
        })
        .collect();
    let fingerprint = plan_result.digest();
    let action_values: Vec<Value> = plan_result
        .actions
        .iter()
        .map(|action| {
            json!({
                "actionId": action.action_id, "process": action.process, "target": action.target, "runs": action.runs,
                "startS": action.start_s, "endS": action.end_s, "verification": action.verification.to_string()
            })
        })
        .collect();
    let plan_sha = sha256_of("poie.plan/0.1", &json!({"fingerprint": fingerprint, "actions": action_values}))
        .map_err(|error| error.to_string())?;
    let mut result = finish_result(
        snapshot,
        variant,
        "plan",
        outcome,
        Some(fingerprint),
        Some(plan_sha),
        causes,
        dependencies,
        actions,
    )?;
    if !warnings.is_empty() {
        result.warnings.extend(warnings);
    }
    result.completion_s = Some(plan_result.completion_s);
    result.residuals = Some(json!({
        "unmetGoals": plan_result.goals_unmet.iter().map(|failure| json!({"goal": failure.goal, "reason": failure.reason})).collect::<Vec<_>>(),
        "completionS": result.completion_s,
        "deadlineS": result.settings.deadline_s,
        "actionCount": result.actions.len(),
    }));
    result.result_sha256 = result_digest(snapshot, variant, &result)?;
    Ok(result)
}

fn plan_outcome(plan: &Plan) -> (&'static str, Vec<String>) {
    if plan.goals_unmet.is_empty() && !plan.goals_met.is_empty() {
        return ("planned", Vec::new());
    }
    let reasons: Vec<&str> = plan.goals_unmet.iter().map(|failure| failure.reason.as_str()).collect();
    let outcome = if reasons.iter().any(|reason| *reason == "unknown state" || reason.starts_with("blocked:")) {
        "unknown"
    } else if reasons.iter().any(|reason| *reason == "search limit" || *reason == "action budget exhausted") {
        "search_limit"
    } else {
        "infeasible"
    };
    let warnings = if reasons.contains(&"deadline missed") { vec!["deadline missed".to_string()] } else { Vec::new() };
    (outcome, warnings)
}

#[allow(clippy::too_many_arguments)]
fn finish_result(
    snapshot: &Snapshot,
    variant: &Variant,
    mode: &str,
    outcome: &str,
    plan_fingerprint: Option<String>,
    plan_sha256: Option<String>,
    root_causes: Vec<CauseView>,
    external_dependencies: Vec<DependencyView>,
    actions: Vec<ActionView>,
) -> Result<VariantResult, String> {
    let counterevidence: Vec<String> = root_causes.iter().map(|cause| cause.detail.clone()).collect();
    let settings = Settings {
        now_s: snapshot.now_s,
        start_s: snapshot.start_s,
        horizon_s: snapshot.horizon_s,
        max_depth: snapshot.max_depth,
        max_actions: snapshot.max_actions,
        deadline_s: variant.deadline_s.or(snapshot.deadline_s),
    };
    let mut result = VariantResult {
        schema: RESULT_SCHEMA.into(),
        schema_version: "0.1".into(),
        result_id: format!("res-{}", variant.id),
        variant_id: variant.id.clone(),
        snapshot_id: snapshot.id.clone(),
        run_id: format!("run-{}", variant.id),
        mode: mode.into(),
        settings,
        outcome: OutcomeKind {
            structure: if mode == "structure" { Some(outcome.into()) } else { None },
            plan: if mode == "plan" { Some(outcome.into()) } else { None },
            caveat: "relative to this static snapshot and bounded solver; not a physical feasibility proof".into(),
        },
        plan_fingerprint,
        plan_sha256,
        completion_s: None,
        actions,
        root_causes,
        external_dependencies,
        result_sha256: String::new(),
        warnings: Vec::new(),
        residuals: None,
        abstained: matches!(outcome, "unknown" | "search_limit"),
        abstain_reason: match outcome {
            "unknown" => Some("unknown_state".into()),
            "search_limit" => Some("search_limit".into()),
            _ => None,
        },
        counterevidence,
    };
    result.result_sha256 = result_digest(snapshot, variant, &result)?;
    Ok(result)
}

pub fn result_digest(snapshot: &Snapshot, variant: &Variant, result: &VariantResult) -> Result<String, String> {
    let payload = json!({
        "snapshotSha256": snapshot.snapshot_sha256,
        "variant": {"patch": variant.patch, "goal": variant.goal, "minVerification": variant.min_verification, "deadlineS": variant.deadline_s},
        "mode": result.mode, "settings": result.settings, "outcome": result.outcome,
        "planSha256": result.plan_sha256, "completionS": result.completion_s,
        "actions": result.actions, "rootCauses": result.root_causes, "externalDependencies": result.external_dependencies,
        "residuals": result.residuals, "abstained": result.abstained, "abstainReason": result.abstain_reason,
        "counterevidence": result.counterevidence,
    });
    sha256_of("poie.variant/0.1", &payload).map_err(|error| error.to_string())
}

pub fn compare(
    snapshot_id: &str,
    results: &[VariantResult],
    baseline: Option<&str>,
    requested: &[String],
) -> Result<CompareResult, String> {
    let mut selected: Vec<&VariantResult> = if requested.is_empty() {
        results.iter().collect()
    } else {
        let mut seen = std::collections::BTreeSet::new();
        let mut warnings = Vec::new();
        let mut selected = Vec::new();
        for id in requested {
            if !seen.insert(id.clone()) {
                warnings.push(format!("duplicate result {id} ignored"));
                continue;
            }
            let Some(result) = results.iter().find(|result| &result.result_id == id) else {
                return Err(format!("unknown_result: {id}"));
            };
            if result.snapshot_id != snapshot_id {
                return Err(format!("invalid_parameter: result {id} belongs to {}", result.snapshot_id));
            }
            selected.push(result);
        }
        return build_compare(snapshot_id, selected, baseline, warnings);
    };
    for result in &selected {
        if result.snapshot_id != snapshot_id {
            return Err(format!("invalid_parameter: result {} belongs to {}", result.result_id, result.snapshot_id));
        }
    }
    selected.sort_by(|left, right| {
        left.outcome
            .plan
            .cmp(&right.outcome.plan)
            .then(left.completion_s.partial_cmp(&right.completion_s).unwrap_or(std::cmp::Ordering::Equal))
            .then(left.result_id.cmp(&right.result_id))
    });
    build_compare(snapshot_id, selected, baseline, Vec::new())
}

fn build_compare(
    snapshot_id: &str,
    selected: Vec<&VariantResult>,
    baseline: Option<&str>,
    warnings: Vec<String>,
) -> Result<CompareResult, String> {
    let baseline_value = baseline
        .and_then(|id| selected.iter().find(|result| result.result_id == id))
        .and_then(|result| result.completion_s);
    let rows = selected.iter().map(|result| CompareRow {
        result_id: result.result_id.clone(), variant_id: result.variant_id.clone(), mode: result.mode.clone(),
        outcome: serde_json::to_value(&result.outcome).unwrap_or(Value::Null), completion_s: result.completion_s,
        plan_fingerprint: result.plan_fingerprint.clone(),
        delta: json!({"completionDeltaS": baseline_value.zip(result.completion_s).map(|(base, value)| value - base), "outcomeChanged": baseline_value.is_some() && baseline != Some(result.result_id.as_str())}),
    }).collect();
    Ok(CompareResult {
        schema: COMPARE_SCHEMA.into(),
        schema_version: "0.1".into(),
        snapshot_id: snapshot_id.into(),
        baseline_result_id: baseline.map(str::to_string),
        rows,
        sort_key: "outcome|completionS|resultId".into(),
        complete: true,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use poie_ontology::model::Model;
    use poie_plan::plan::GoalFailure;

    fn model() -> Model {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/base-01/model.toml");
        Model::load(&path).expect("demo model")
    }

    fn snapshot(model: &Model) -> Snapshot {
        Snapshot::create(
            SnapshotInput {
                label: "base".into(),
                model_sha256: poie_objects_model_digest(model).unwrap(),
                evidence_sha256: None,
                goal: "service:potable-water".into(),
                min_verification: "demonstrated".into(),
                horizon_s: 24.0,
                now_s: 0.0,
                start_s: 0.0,
                deadline_s: None,
                max_depth: 12,
                max_actions: 12,
                seed: 7,
            },
            "snap-1".into(),
            1,
            "Ada".into(),
        )
        .unwrap()
    }

    fn variant(snapshot: &Snapshot, patch: PatchDto) -> Variant {
        Variant {
            schema: VARIANT_SCHEMA.into(),
            schema_version: "0.1".into(),
            id: "var-1".into(),
            snapshot_id: snapshot.id.clone(),
            label: "v".into(),
            description: String::new(),
            patch,
            goal: None,
            min_verification: None,
            deadline_s: None,
            created_at: 1,
            created_by: "Ada".into(),
        }
    }

    #[test]
    fn snapshot_digest_excludes_identity_fields() {
        let model = model();
        let a = snapshot(&model);
        let mut b = a.clone();
        b.id = "other".into();
        b.created_at = 2;
        b.created_by = "B".into();
        b.seed = 9;
        assert_eq!(a.snapshot_sha256, snapshot_digest(&b).unwrap());
        assert_ne!(a.snapshot_sha256, snapshot_digest(&a).unwrap_or_default() + "x");
    }

    #[test]
    fn structural_variant_is_deterministic_and_caveated() {
        let model = model();
        let snapshot = snapshot(&model);
        let variant = variant(&snapshot, PatchDto::default());
        let first = run_structure(&model, &snapshot, &variant).unwrap();
        let second = run_structure(&model, &snapshot, &variant).unwrap();
        assert_eq!(first.result_sha256, second.result_sha256);
        assert!(first.outcome.structure.is_some());
        assert!(first.outcome.caveat.contains("not a physical"));
        assert_eq!(first.counterevidence.len(), first.root_causes.len());
    }

    #[test]
    fn patch_rejects_unknown_asset_status() {
        let mut patch = PatchDto::default();
        patch.asset_status.insert("pump-p1".into(), "exploded".into());
        assert!(patch.to_patch().is_err());
    }

    #[test]
    fn plan_outcome_never_marks_unmet_as_planned() {
        let failure = GoalFailure { goal: "service:x".into(), reason: "deadline missed".into() };
        let plan = Plan {
            plan_id: "p".into(),
            start_s: 0.0,
            horizon_s: 1.0,
            actions: vec![],
            goals_met: vec![],
            goals_unmet: vec![failure],
            completion_s: 0.0,
        };
        assert_eq!(plan_outcome(&plan).0, "infeasible");
        let search =
            Plan { goals_unmet: vec![GoalFailure { goal: "s".into(), reason: "search limit".into() }], ..plan.clone() };
        assert_eq!(plan_outcome(&search).0, "search_limit");
    }

    #[test]
    fn compare_rejects_cross_snapshot() {
        let model = model();
        let snapshot = snapshot(&model);
        let variant = variant(&snapshot, PatchDto::default());
        let result = run_structure(&model, &snapshot, &variant).unwrap();
        let mut other = result.clone();
        other.snapshot_id = "other".into();
        assert!(compare("snap-1", &[result, other], None, &[]).is_err());
    }

    #[test]
    fn plan_result_carries_residuals_and_abstention_fields() {
        let model = model();
        let mut snapshot = snapshot(&model);
        snapshot.horizon_s = 86_400.0;
        snapshot.snapshot_sha256 = snapshot_digest(&snapshot).unwrap();
        let variant = variant(&snapshot, PatchDto::default());
        let result = run_plan(&model, &snapshot, &variant).unwrap();
        assert!(result.residuals.is_some());
        assert!(!result.abstained);
        assert!(result.residuals.unwrap()["completionS"].is_number());
    }
}
