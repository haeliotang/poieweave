//! Phase C1 what-if commands: static snapshots, Patch variants, compare.
//!
//! The source model is read-only here. Results are written only to the
//! `whatif_*` tables; no live/SAP/federation/approval path is touched.

use poie_governance::{subject_digest, ApprovalToken};
use poie_plan::sap::{plan_to_actions_at, preview_actions, AssumedContext};
use poie_sim::timed::{TimedCase, TimedRunner, TimedScenario};
use poie_whatif::promotion::{
    proposed_action_sha256, PromotionMandate, PROMOTE_POLICY_VERSION, PROPOSED_ACTION_SCHEMA,
};
use poie_whatif::promotion::{PromotionCommit, PromotionEvidence, PromotionProposal, ProposedAction};
use poie_whatif::timed::{
    compare_timed, TimedCompareResult, TimedResult, TimedSnapshot, TimedSnapshotInput, TimedVariant, TimedVariantInput,
    TIMED_SNAPSHOT_SCHEMA,
};
use poie_whatif::{
    compare, run_plan, run_structure, CompareResult, PatchDto, Snapshot, SnapshotInput, Variant, VariantResult,
    VARIANT_SCHEMA,
};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::auth;
use crate::{sandbox, AppState};

fn random_id(prefix: &str) -> Result<String, String> {
    let mut bytes = [0u8; 12];
    getrandom::getrandom(&mut bytes).map_err(|error| format!("random id: {error}"))?;
    Ok(format!("{prefix}-{}", hex::encode(bytes)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VariantCreateInput {
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

#[tauri::command]
pub fn whatif_model_digest(state: State<'_, AppState>) -> Result<String, String> {
    auth::require(&state.session)?;
    poie_objects::model_digest(&state.model).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn whatif_snapshot_create(input: SnapshotInput, state: State<'_, AppState>) -> Result<Snapshot, String> {
    let operator = auth::require(&state.session)?;
    let id = random_id("snap")?;
    let snapshot = Snapshot::create(input, id.clone(), auth::now_ms(), operator.display_name.clone())?;
    snapshot.verify_model(&state.model)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let json = serde_json::to_string(&snapshot).map_err(|error| error.to_string())?;
    db.save_whatif_snapshot(
        &id,
        &snapshot.schema_version,
        &operator.session_id,
        &json,
        &snapshot.snapshot_sha256,
        auth::now_ms(),
    )?;
    db.append_audit(auth::now_ms(), &operator.display_name, "whatif.snapshot", &format!("{id} {}", snapshot.goal))?;
    Ok(snapshot)
}

#[tauri::command]
pub fn whatif_snapshot_list(limit: Option<u32>, state: State<'_, AppState>) -> Result<Vec<Snapshot>, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    db.whatif_snapshots(limit.unwrap_or(20).clamp(1, 100))?
        .into_iter()
        .map(|json| serde_json::from_str::<Snapshot>(&json).map_err(|error| error.to_string()))
        .collect()
}

#[tauri::command]
pub fn whatif_variant_create(input: VariantCreateInput, state: State<'_, AppState>) -> Result<Variant, String> {
    let operator = auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let snapshot_json =
        db.whatif_snapshot(&input.snapshot_id)?.ok_or_else(|| format!("unknown_snapshot: {}", input.snapshot_id))?;
    let snapshot: Snapshot = serde_json::from_str(&snapshot_json).map_err(|error| error.to_string())?;
    snapshot.verify_model(&state.model)?;
    let variant = Variant {
        schema: poie_whatif::VARIANT_SCHEMA.into(),
        schema_version: "0.1".into(),
        id: random_id("var")?,
        snapshot_id: snapshot.id.clone(),
        label: input.label,
        description: input.description,
        patch: input.patch,
        goal: input.goal,
        min_verification: input.min_verification,
        deadline_s: input.deadline_s,
        created_at: auth::now_ms(),
        created_by: operator.display_name.clone(),
    };
    let json = serde_json::to_string(&variant).map_err(|error| error.to_string())?;
    db.save_whatif_variant(&variant.id, &variant.snapshot_id, VARIANT_SCHEMA, &json, auth::now_ms())?;
    db.append_audit(
        auth::now_ms(),
        &operator.display_name,
        "whatif.variant",
        &format!("{} {}", variant.id, variant.snapshot_id),
    )?;
    Ok(variant)
}

#[tauri::command]
pub fn whatif_variant_list(snapshot_id: String, state: State<'_, AppState>) -> Result<Vec<Variant>, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    db.whatif_variants(&snapshot_id)?
        .into_iter()
        .map(|json| serde_json::from_str::<Variant>(&json).map_err(|error| error.to_string()))
        .collect()
}

#[tauri::command]
pub fn whatif_variant_run(
    variant_id: String,
    mode: String,
    state: State<'_, AppState>,
) -> Result<VariantResult, String> {
    let operator = auth::require(&state.session)?;
    if mode != "structure" && mode != "plan" {
        return Err("invalid_parameter: mode must be structure or plan".into());
    }
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let variant_json = db.whatif_variant(&variant_id)?.ok_or_else(|| format!("unknown_variant: {variant_id}"))?;
    let variant: Variant = serde_json::from_str(&variant_json).map_err(|error| error.to_string())?;
    let snapshot_json = db
        .whatif_snapshot(&variant.snapshot_id)?
        .ok_or_else(|| format!("unknown_snapshot: {}", variant.snapshot_id))?;
    let snapshot: Snapshot = serde_json::from_str(&snapshot_json).map_err(|error| error.to_string())?;
    let result = if mode == "structure" {
        run_structure(&state.model, &snapshot, &variant)?
    } else {
        run_plan(&state.model, &snapshot, &variant)?
    };
    let settings = serde_json::to_string(&result.settings).map_err(|error| error.to_string())?;
    let json = serde_json::to_string(&result).map_err(|error| error.to_string())?;
    db.save_whatif_result(
        &result.result_id,
        &result.variant_id,
        &result.snapshot_id,
        &result.schema_version,
        &result.mode,
        &settings,
        &json,
        &result.result_sha256,
        auth::now_ms(),
    )?;
    db.append_audit(
        auth::now_ms(),
        &operator.display_name,
        "whatif.run",
        &format!("{} {} {}", result.result_id, result.mode, result.result_sha256),
    )?;
    Ok(result)
}

#[tauri::command]
pub fn whatif_result_get(result_id: String, state: State<'_, AppState>) -> Result<VariantResult, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let json = db.whatif_result(&result_id)?.ok_or_else(|| format!("unknown_result: {result_id}"))?;
    serde_json::from_str(&json).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn whatif_result_list(
    snapshot_id: String,
    variant_id: Option<String>,
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<VariantResult>, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    db.whatif_results(&snapshot_id, variant_id.as_deref(), limit.unwrap_or(50).clamp(1, 100))?
        .into_iter()
        .map(|json| serde_json::from_str::<VariantResult>(&json).map_err(|error| error.to_string()))
        .collect()
}

#[tauri::command]
pub fn whatif_compare(
    snapshot_id: String,
    baseline_result_id: Option<String>,
    result_ids: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<CompareResult, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let results: Vec<VariantResult> = db
        .whatif_results(&snapshot_id, None, 200)?
        .into_iter()
        .map(|json| serde_json::from_str::<VariantResult>(&json).map_err(|error| error.to_string()))
        .collect::<Result<_, _>>()?;
    compare(&snapshot_id, &results, baseline_result_id.as_deref(), result_ids.unwrap_or_default().as_slice())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimedScenarioInfo {
    pub scenario_sha256: String,
    pub scenario_path: String,
    pub model_sha256: String,
    pub description: String,
}

fn timed_scenario_sha(scenario_text: &str) -> String {
    let mut bytes = Vec::with_capacity(64 + scenario_text.len());
    bytes.extend_from_slice(poie_sim::timed::TIMED_SCENARIO_SCHEMA.as_bytes());
    bytes.push(b'\n');
    bytes.extend_from_slice(scenario_text.as_bytes());
    poie_digest::hex_sha256(&bytes)
}

fn model_sha256(model: &poie_ontology::model::Model) -> Result<String, String> {
    poie_objects::model_digest(model).map_err(|error| error.to_string())
}

/// Load a sandboxed timed scenario + its model without path escapes.
fn load_timed_scenario(
    state: &AppState,
    scenario_path: &str,
) -> Result<(TimedScenario, String, String, String), String> {
    let text = sandbox::read(&state.sandbox_root, scenario_path).map_err(|error| format!("io_error: {error}"))?;
    let parsed: toml::Value = toml::from_str(&text).map_err(|error| format!("scenario_mismatch: {error}"))?;
    let model_relative = parsed
        .get("model")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| "scenario_mismatch: missing model".to_string())?;
    let parent = std::path::Path::new(scenario_path).parent().and_then(|path| path.to_str()).unwrap_or_default();
    let model_text = sandbox::read_inside(&state.sandbox_root, parent, model_relative)
        .map_err(|error| format!("io_error: {error}"))?;
    let doc: poie_ontology::model::ModelDoc =
        toml::from_str(&model_text).map_err(|error| format!("model_mismatch: {error}"))?;
    let model = poie_ontology::model::Model::from_doc(doc).map_err(|error| format!("model_mismatch: {error}"))?;
    let scenario_sha = timed_scenario_sha(&text);
    let model_sha = model_sha256(&model)?;
    let expected = model_sha256(&state.model)?;
    if model_sha != expected {
        return Err("model_mismatch: scenario model is not the host SourceModel".into());
    }
    let scenario = TimedScenario::from_source(&text, model).map_err(|error| format!("scenario_mismatch: {error}"))?;
    Ok((scenario, scenario_sha, model_sha, text))
}

#[tauri::command]
pub fn whatif_timed_scenario_import(path: String, state: State<'_, AppState>) -> Result<TimedScenarioInfo, String> {
    let operator = auth::require(&state.session)?;
    let (scenario, scenario_sha256, model_sha256, text) = load_timed_scenario(&state, &path)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    db.save_timed_scenario(
        &scenario_sha256,
        &path,
        poie_sim::timed::TIMED_SCENARIO_SCHEMA,
        &model_sha256,
        &scenario.description,
        auth::now_ms(),
    )?;
    db.append_audit(
        auth::now_ms(),
        &operator.display_name,
        "whatif.scenario.import",
        &format!("{scenario_sha256} {path} {} bytes", text.len()),
    )?;
    Ok(TimedScenarioInfo { scenario_sha256, scenario_path: path, model_sha256, description: scenario.description })
}

#[tauri::command]
pub fn whatif_timed_scenario_list(state: State<'_, AppState>) -> Result<Vec<TimedScenarioInfo>, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    Ok(db
        .timed_scenarios(50)?
        .into_iter()
        .map(|(scenario_sha256, scenario_path, model_sha256, description, _)| TimedScenarioInfo {
            scenario_sha256,
            scenario_path,
            model_sha256,
            description,
        })
        .collect())
}

#[tauri::command]
pub fn whatif_timed_snapshot_create(
    input: TimedSnapshotInput,
    state: State<'_, AppState>,
) -> Result<TimedSnapshot, String> {
    let operator = auth::require(&state.session)?;
    let stored = {
        let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
        db.timed_scenario(&input.scenario_sha256)?
    };
    if stored.is_none() {
        return Err(format!("unknown_scenario: import {} first", input.scenario_sha256));
    }
    let (_scenario, actual_sha, actual_model, _) = load_timed_scenario(&state, &input.scenario_path)?;
    if actual_sha != input.scenario_sha256 || actual_model != input.model_sha256 {
        return Err("stale_scenario: scenario/model changed since import".into());
    }
    let snapshot = TimedSnapshot::create(input, random_id("tsnap")?, auth::now_ms(), operator.display_name.clone())?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let json = serde_json::to_string(&snapshot).map_err(|error| error.to_string())?;
    db.save_timed_snapshot(&snapshot.id, TIMED_SNAPSHOT_SCHEMA, &json, &snapshot.snapshot_sha256, auth::now_ms())?;
    db.append_audit(
        auth::now_ms(),
        &operator.display_name,
        "whatif.timed.snapshot",
        &format!("{} {}", snapshot.id, snapshot.snapshot_sha256),
    )?;
    Ok(snapshot)
}

fn load_timed_snapshot(state: &AppState, id: &str) -> Result<TimedSnapshot, String> {
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let json = db.timed_snapshot(id)?.ok_or_else(|| format!("unknown_snapshot: {id}"))?;
    serde_json::from_str(&json).map_err(|error| error.to_string())
}

fn load_timed_variant(state: &AppState, id: &str) -> Result<TimedVariant, String> {
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let json = db.timed_variant(id)?.ok_or_else(|| format!("unknown_variant: {id}"))?;
    serde_json::from_str(&json).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn whatif_timed_variant_create(
    input: TimedVariantInput,
    state: State<'_, AppState>,
) -> Result<TimedVariant, String> {
    let operator = auth::require(&state.session)?;
    let snapshot = load_timed_snapshot(&state, &input.snapshot_id)?;
    let (scenario, _, _, _) = load_timed_scenario(&state, &snapshot.scenario_path)?;
    if input.start_mode == "at" {
        let start = input.start_s.ok_or_else(|| "invalid_parameter: startS required".to_string())?;
        if start > snapshot.horizon_s {
            return Err("invalid_parameter: startS exceeds horizonS".into());
        }
    }
    let deadline = input.deadline_s.or(snapshot.deadline_s);
    if input.start_mode == "at" {
        if let Some(deadline) = deadline {
            if deadline <= input.start_s.unwrap_or(0.0) || deadline > snapshot.horizon_s {
                return Err("invalid_parameter: deadlineS must be inside (startS, horizonS]".into());
            }
        }
    }
    for (resource, watts) in &input.storage_discharge_w {
        let Some(original) = scenario.storage_discharge_w.get(resource) else {
            return Err(format!("invalid_parameter: unknown storage resource {resource}"));
        };
        if *watts > *original + 1e-9 {
            return Err(format!("power_limit_exceeded: {resource} exceeds scenario limit"));
        }
    }
    let variant = TimedVariant::create(input, random_id("tvar")?, auth::now_ms(), operator.display_name.clone())?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let json = serde_json::to_string(&variant).map_err(|error| error.to_string())?;
    db.save_timed_variant(&variant.id, &variant.snapshot_id, "0.1", &json, auth::now_ms())?;
    db.append_audit(
        auth::now_ms(),
        &operator.display_name,
        "whatif.timed.variant",
        &format!("{} {}", variant.id, variant.snapshot_id),
    )?;
    Ok(variant)
}

#[tauri::command]
pub fn whatif_timed_variant_list(snapshot_id: String, state: State<'_, AppState>) -> Result<Vec<TimedVariant>, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    db.whatif_timed_variants(&snapshot_id)?
        .into_iter()
        .map(|json| serde_json::from_str::<TimedVariant>(&json).map_err(|error| error.to_string()))
        .collect()
}

#[tauri::command]
pub fn whatif_timed_run(variant_id: String, state: State<'_, AppState>) -> Result<TimedResult, String> {
    let operator = auth::require(&state.session)?;
    let variant = load_timed_variant(&state, &variant_id)?;
    let snapshot = load_timed_snapshot(&state, &variant.snapshot_id)?;
    let (mut scenario, scenario_sha, model_sha, _) = load_timed_scenario(&state, &snapshot.scenario_path)?;
    if scenario_sha != snapshot.scenario_sha256 || model_sha != snapshot.model_sha256 {
        return Err("stale_scenario: scenario/model changed since snapshot".into());
    }
    for (resource, watts) in &variant.storage_discharge_w {
        let Some(limit) = scenario.storage_discharge_w.get_mut(resource) else {
            return Err(format!("invalid_parameter: unknown storage resource {resource}"));
        };
        if *watts > *limit + 1e-9 {
            return Err(format!("power_limit_exceeded: {resource} exceeds scenario limit"));
        }
        *limit = *watts;
    }
    let start_s = match variant.start_mode.as_str() {
        "beginning" => 0.0,
        "at" => variant.start_s.ok_or_else(|| "invalid_parameter: startS required".to_string())?,
        other => return Err(format!("invalid_parameter: unknown startMode {other:?}")),
    };
    if start_s > snapshot.horizon_s {
        return Err("invalid_parameter: startS exceeds horizonS".into());
    }
    let deadline_s = variant.deadline_s.or(snapshot.deadline_s);
    let goal = match snapshot.goal.split_once(':') {
        Some(("service", id)) if !id.is_empty() => poie_ontology::graph::Goal::Service(id.to_string()),
        Some(("restore", id)) if !id.is_empty() => poie_ontology::graph::Goal::Restore(id.to_string()),
        _ => return Err("invalid_parameter: snapshot goal".into()),
    };
    let min_verification: poie_ontology::model::Verification =
        serde_json::from_value(serde_json::json!(snapshot.min_verification)).map_err(|error| error.to_string())?;
    let case = TimedCase { goal, start_s, deadline_s, horizon_s: snapshot.horizon_s, min_verification };
    let outcome = TimedRunner::new(scenario).evaluate(&case).map_err(|error| format!("scenario_mismatch: {error}"))?;
    let result = TimedResult::create(
        &snapshot,
        &variant,
        random_id("tres")?,
        format!("whatif-timed-{}", auth::now_ms()),
        start_s,
        deadline_s,
        outcome.verdict.as_str(),
        outcome.completion_s,
        outcome.warnings,
        auth::now_ms(),
    )?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let settings = serde_json::to_string(&result.settings).map_err(|error| error.to_string())?;
    let json = serde_json::to_string(&result).map_err(|error| error.to_string())?;
    db.save_timed_result(
        &result.result_id,
        &result.variant_id,
        &result.snapshot_id,
        &result.schema_version,
        &result.mode,
        &settings,
        &json,
        &result.result_sha256,
        auth::now_ms(),
    )?;
    db.append_audit(
        auth::now_ms(),
        &operator.display_name,
        "whatif.timed.run",
        &format!("{} {} {}", result.result_id, result.outcome.timed, result.result_sha256),
    )?;
    Ok(result)
}

#[tauri::command]
pub fn whatif_timed_result_get(result_id: String, state: State<'_, AppState>) -> Result<TimedResult, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let json = db.timed_result(&result_id)?.ok_or_else(|| format!("unknown_result: {result_id}"))?;
    serde_json::from_str(&json).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn whatif_timed_result_list(
    snapshot_id: String,
    variant_id: Option<String>,
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<TimedResult>, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    db.timed_results(&snapshot_id, variant_id.as_deref(), limit.unwrap_or(50).clamp(1, 100))?
        .into_iter()
        .map(|json| serde_json::from_str::<TimedResult>(&json).map_err(|error| error.to_string()))
        .collect()
}

#[tauri::command]
pub fn whatif_timed_compare(
    snapshot_id: String,
    baseline_result_id: Option<String>,
    result_ids: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<TimedCompareResult, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let mut results: Vec<TimedResult> = db
        .timed_results(&snapshot_id, None, 200)?
        .into_iter()
        .map(|json| serde_json::from_str::<TimedResult>(&json).map_err(|error| error.to_string()))
        .collect::<Result<_, _>>()?;
    if let Some(ids) = result_ids {
        results.retain(|result| ids.contains(&result.result_id));
    }
    compare_timed(&snapshot_id, baseline_result_id.as_deref(), results)
}

fn promotion_state_version() -> std::collections::BTreeMap<String, u64> {
    std::collections::BTreeMap::from([("site-a".to_string(), 4)])
}

fn load_promotion(state: &AppState, proposal_id: &str) -> Result<PromotionProposal, String> {
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let json = db.promotion_proposal(proposal_id)?.ok_or_else(|| format!("unknown_proposal: {proposal_id}"))?;
    let mut proposal: PromotionProposal = serde_json::from_str(&json).map_err(|error| error.to_string())?;
    if let Some((status, _, _)) = db.promotion_status(proposal_id)? {
        proposal.status = status;
    }
    Ok(proposal)
}

fn token_for(db: &crate::db::Db, id: &str) -> Result<ApprovalToken, String> {
    db.approval_token(id)?.ok_or_else(|| format!("approval_required: missing token {id}"))
}

fn plan_for_promotion(
    model: &poie_ontology::model::Model,
    snapshot: &Snapshot,
    variant: &Variant,
) -> Result<poie_plan::plan::Plan, String> {
    let patch = variant.patch.to_patch()?;
    let model = model.patched(&patch).map_err(|error| format!("invalid_patch: {error}"))?;
    let goal_text = variant.goal.as_deref().unwrap_or(&snapshot.goal);
    let goal = match goal_text.split_once(':') {
        Some(("service", id)) if !id.is_empty() => poie_ontology::graph::Goal::Service(id.to_string()),
        Some(("restore", id)) if !id.is_empty() => poie_ontology::graph::Goal::Restore(id.to_string()),
        _ => return Err("invalid_parameter: goal".into()),
    };
    let min_verification: poie_ontology::model::Verification = serde_json::from_value(serde_json::json!(variant
        .min_verification
        .as_deref()
        .unwrap_or(&snapshot.min_verification)))
    .map_err(|error| error.to_string())?;
    let deadline = variant.deadline_s.or(snapshot.deadline_s);
    poie_plan::plan::plan(&poie_plan::plan::PlanningRequest {
        model: &model,
        now_s: snapshot.now_s,
        horizon_s: snapshot.horizon_s,
        goals: vec![poie_plan::plan::PlanningGoal { goal, deadline_s: deadline, priority: 10 }],
        min_verification,
        max_actions: snapshot.max_actions,
    })
    .map_err(|error| format!("plan_failed: {error}"))
}

#[tauri::command]
pub fn whatif_promote_propose(result_id: String, state: State<'_, AppState>) -> Result<PromotionProposal, String> {
    let operator = auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    if db.active_promotion_for_result(&result_id)?.is_some() {
        return Err("duplicate_proposal: an active proposal already exists for this result".into());
    }
    let result_json = db.whatif_result(&result_id)?.ok_or_else(|| format!("unknown_result: {result_id}"))?;
    let result: VariantResult = serde_json::from_str(&result_json).map_err(|error| error.to_string())?;
    if result.mode != "plan" || result.outcome.plan.as_deref() != Some("planned") || result.actions.is_empty() {
        return Err("not_promotable: only planned C1 results with actions can be promoted".into());
    }
    let result_sha256 = result.result_sha256.clone();
    let variant_json =
        db.whatif_variant(&result.variant_id)?.ok_or_else(|| format!("unknown_variant: {}", result.variant_id))?;
    let snapshot_json =
        db.whatif_snapshot(&result.snapshot_id)?.ok_or_else(|| format!("unknown_snapshot: {}", result.snapshot_id))?;
    let variant: Variant = serde_json::from_str(&variant_json).map_err(|error| error.to_string())?;
    let snapshot: Snapshot = serde_json::from_str(&snapshot_json).map_err(|error| error.to_string())?;
    snapshot.verify_model(&state.model)?;
    let variant_sha = poie_digest::sha256_of("poie.whatif-variant/0.1", &variant).map_err(|error| error.to_string())?;
    let planned = plan_for_promotion(&state.model, &snapshot, &variant)?;
    if Some(planned.digest()) != result.plan_fingerprint {
        return Err("plan_mismatch: stored plan fingerprint no longer matches".into());
    }
    let now = auth::now_ms();
    let state_version = promotion_state_version();
    let actions = plan_to_actions_at(&planned, snapshot.now_s, now, &state_version)
        .map_err(|error| format!("sap_rejected: {error}"))?;
    if actions.is_empty() {
        return Err("not_promotable: plan has no actions".into());
    }
    let predicates = actions.iter().flat_map(|action| action.precondition.predicates.clone()).collect();
    let preview = preview_actions(
        &actions,
        &AssumedContext {
            now_ms: now,
            holds_command_ownership: true,
            reservations_held: true,
            predicates_holding: predicates,
            state_version: state_version.clone(),
        },
    )
    .map_err(|error| format!("sap_rejected: {error}"))?;
    if !preview.rejections.is_empty() {
        return Err(format!("sap_rejected: {}", preview.rejections.join("; ")));
    }
    let proposal_id = random_id("prom")?;
    let token_id = random_id("ptok")?;
    let proposal_scope = format!("promote:{}:{}", result.snapshot_id, result.result_id);
    let expires = now + 30 * 60 * 1000;
    let mandate = PromotionMandate {
        mandate_ref: format!("operator:{}:{}", operator.session_id, proposal_id),
        granted_by: operator.display_name.clone(),
        granted_to: operator.display_name.clone(),
        scope: proposal_scope.clone(),
        issued_at: now,
        expires_at: expires,
        revocable: true,
    };
    let proposal = PromotionProposal::create(
        result.result_id.clone(),
        result.snapshot_id.clone(),
        result.variant_id.clone(),
        result_sha256,
        snapshot.snapshot_sha256.clone(),
        variant_sha,
        &planned,
        snapshot.model_sha256.clone(),
        state_version,
        actions,
        preview,
        Some(poie_whatif::promotion::PromotionAssessment {
            residuals: result.residuals.clone(),
            abstained: result.abstained,
            abstain_reason: result.abstain_reason.clone(),
            counterevidence: result.counterevidence.clone(),
        }),
        Some(mandate),
        proposal_id.clone(),
        token_id.clone(),
        now,
        operator.display_name.clone(),
    )?;
    let token_args = serde_json::json!({"scope": proposal_scope, "resultId": result.result_id, "proposalSha256": proposal.proposal_sha256, "mandateRef": proposal.mandate.as_ref().map(|m| m.mandate_ref.clone())});
    let token_subject =
        subject_digest("whatif.promote", &token_args, &operator.session_id, PROMOTE_POLICY_VERSION, expires)
            .map_err(|error| error.to_string())?;
    let token = ApprovalToken {
        id: token_id.clone(),
        session_id: operator.session_id.clone(),
        run_id: proposal_id.clone(),
        call_id: String::new(),
        tool: "whatif.promote".into(),
        tool_version: "1.0.0".into(),
        subject_digest: token_subject,
        policy_version: PROMOTE_POLICY_VERSION.into(),
        args_json_redacted: serde_json::to_string(&token_args).map_err(|error| error.to_string())?,
        impact_json: serde_json::json!({"actions": proposal.actions.len(), "scope": proposal_scope}).to_string(),
        citations_json: serde_json::json!([{"kind":"plan","id":proposal.plan_fingerprint}]).to_string(),
        state: "pending".into(),
        created_at_ms: now,
        decided_at_ms: None,
        started_at_ms: None,
        expires_at_ms: expires,
    };
    db.save_approval_token(&token)?;
    db.save_promotion_proposal(
        &proposal_id,
        &result.result_id,
        &result.snapshot_id,
        &result.variant_id,
        &serde_json::to_string(&proposal).map_err(|error| error.to_string())?,
        &proposal.proposal_sha256,
        "pending",
        &token_id,
        now,
    )?;
    db.append_audit(
        now,
        &operator.display_name,
        "promotion.proposed",
        &format!("{proposal_id} {result_id} {proposal_scope}"),
    )?;
    Ok(proposal)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotionReplayReport {
    pub ok: bool,
    pub plan_match: bool,
    pub actions_match: bool,
    pub inputs_match: bool,
    pub expected_plan_sha256: String,
    pub actual_plan_sha256: String,
    pub expected_actions_sha256: String,
    pub actual_actions_sha256: String,
    pub expected_inputs_sha256: String,
    pub actual_inputs_sha256: String,
}

#[tauri::command]
pub fn whatif_promote_replay(proposal_id: String, state: State<'_, AppState>) -> Result<PromotionReplayReport, String> {
    auth::require(&state.session)?;
    let proposal = load_promotion(&state, &proposal_id)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let result_json =
        db.whatif_result(&proposal.result_id)?.ok_or_else(|| format!("unknown_result: {}", proposal.result_id))?;
    let result: VariantResult = serde_json::from_str(&result_json).map_err(|error| error.to_string())?;
    let variant_json =
        db.whatif_variant(&proposal.variant_id)?.ok_or_else(|| format!("unknown_variant: {}", proposal.variant_id))?;
    let snapshot_json = db
        .whatif_snapshot(&proposal.snapshot_id)?
        .ok_or_else(|| format!("unknown_snapshot: {}", proposal.snapshot_id))?;
    drop(db);
    let variant: Variant = serde_json::from_str(&variant_json).map_err(|error| error.to_string())?;
    let snapshot: Snapshot = serde_json::from_str(&snapshot_json).map_err(|error| error.to_string())?;
    snapshot.verify_model(&state.model)?;
    let planned = plan_for_promotion(&state.model, &snapshot, &variant)?;
    let actual_plan_sha = poie_whatif::promotion::promotion_plan_sha256(&planned)?;
    let actual_actions = plan_to_actions_at(&planned, snapshot.now_s, proposal.now_ms, &proposal.state_version)
        .map_err(|error| format!("sap_rejected: {error}"))?;
    let actual_actions_sha = poie_whatif::promotion::proposed_actions_sha256(&actual_actions)?;
    let variant_sha = poie_digest::sha256_of("poie.whatif-variant/0.1", &variant).map_err(|error| error.to_string())?;
    let actual_inputs_sha = poie_whatif::promotion::promotion_inputs_sha256(
        &snapshot.snapshot_sha256,
        &variant_sha,
        &result.result_sha256,
    )?;
    let plan_match = actual_plan_sha == proposal.plan_sha256;
    let actions_match = actual_actions_sha == proposal.actions_sha256;
    let inputs_match = actual_inputs_sha == proposal.inputs_sha256;
    Ok(PromotionReplayReport {
        ok: plan_match && actions_match && inputs_match,
        plan_match,
        actions_match,
        inputs_match,
        expected_plan_sha256: proposal.plan_sha256,
        actual_plan_sha256: actual_plan_sha,
        expected_actions_sha256: proposal.actions_sha256,
        actual_actions_sha256: actual_actions_sha,
        expected_inputs_sha256: proposal.inputs_sha256,
        actual_inputs_sha256: actual_inputs_sha,
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotionAppeal {
    pub id: i64,
    pub operator: String,
    pub reason: String,
    pub status: String,
    pub created_at: i64,
}

#[tauri::command]
pub fn whatif_promote_recuse(proposal_id: String, reason: String, state: State<'_, AppState>) -> Result<(), String> {
    let operator = auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    if db.promotion_proposal(&proposal_id)?.is_none() {
        return Err(format!("unknown_proposal: {proposal_id}"));
    }
    db.save_promotion_conflict(&proposal_id, &operator.display_name, &reason, auth::now_ms())?;
    db.append_audit(auth::now_ms(), &operator.display_name, "promotion.recused", &format!("{proposal_id} {reason}"))?;
    Ok(())
}

#[tauri::command]
pub fn whatif_promote_appeal(proposal_id: String, reason: String, state: State<'_, AppState>) -> Result<(), String> {
    let operator = auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    if db.promotion_proposal(&proposal_id)?.is_none() {
        return Err(format!("unknown_proposal: {proposal_id}"));
    }
    db.save_promotion_appeal(&proposal_id, &operator.display_name, &reason, auth::now_ms())?;
    db.append_audit(auth::now_ms(), &operator.display_name, "promotion.appealed", &format!("{proposal_id} {reason}"))?;
    Ok(())
}

#[tauri::command]
pub fn whatif_promote_appeals(proposal_id: String, state: State<'_, AppState>) -> Result<Vec<PromotionAppeal>, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    Ok(db
        .promotion_appeals(&proposal_id)?
        .into_iter()
        .map(|(id, operator, reason, status, created_at)| PromotionAppeal { id, operator, reason, status, created_at })
        .collect())
}

#[tauri::command]
pub fn whatif_promote_policy(state: State<'_, AppState>) -> Result<bool, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    db.promotion_four_eyes()
}

#[tauri::command]
pub fn whatif_promote_set_policy(enabled: bool, state: State<'_, AppState>) -> Result<bool, String> {
    let operator = auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    db.set_promotion_four_eyes(enabled, auth::now_ms())?;
    db.append_audit(auth::now_ms(), &operator.display_name, "promotion.policy", &format!("fourEyes={enabled}"))?;
    Ok(enabled)
}

#[tauri::command]
pub fn whatif_promote_get(proposal_id: String, state: State<'_, AppState>) -> Result<PromotionProposal, String> {
    auth::require(&state.session)?;
    load_promotion(&state, &proposal_id)
}

#[tauri::command]
pub fn whatif_promote_list(limit: Option<u32>, state: State<'_, AppState>) -> Result<Vec<PromotionProposal>, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    db.promotion_proposals(limit.unwrap_or(20).clamp(1, 100))?
        .into_iter()
        .map(|(status, json)| {
            let mut proposal: PromotionProposal = serde_json::from_str(&json).map_err(|error| error.to_string())?;
            proposal.status = status;
            Ok(proposal)
        })
        .collect()
}

fn second_approver_allowed(four_eyes: bool, requested_by: &str, approver: &str) -> bool {
    four_eyes && requested_by != approver
}

fn mandate_allows(proposal: &PromotionProposal, now: i64) -> Result<(), String> {
    let Some(mandate) = &proposal.mandate else {
        return Ok(());
    };
    if mandate.scope != proposal.scope {
        return Err("scope_denied: mandate scope does not match proposal".into());
    }
    if now > mandate.expires_at {
        return Err("approval_expired: mandate expired".into());
    }
    Ok(())
}

fn decide_promotion(
    approve: bool,
    proposal_id: &str,
    state: &AppState,
    operator: &crate::auth::Session,
) -> Result<PromotionProposal, String> {
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let proposal_json =
        db.promotion_proposal(proposal_id)?.ok_or_else(|| format!("unknown_proposal: {proposal_id}"))?;
    let mut proposal: PromotionProposal = serde_json::from_str(&proposal_json).map_err(|error| error.to_string())?;
    if db.promotion_conflict(proposal_id, &operator.display_name)? {
        return Err("recused: operator declared a conflict for this proposal".into());
    }
    if db.promotion_four_eyes()? && proposal.requested_by == operator.display_name {
        return Err("separation_of_duties: four-eyes policy requires a different approver".into());
    }
    mandate_allows(&proposal, auth::now_ms())?;
    let token = token_for(&db, &proposal.approval_token_id)?;
    let second_approver =
        second_approver_allowed(db.promotion_four_eyes()?, &proposal.requested_by, &operator.display_name);
    if (!second_approver && token.session_id != operator.session_id)
        || token.tool != "whatif.promote"
        || token.run_id != proposal.proposal_id
    {
        return Err("approval_mismatch: token does not belong to this proposal/session".into());
    }
    let target = if approve { "approved" } else { "rejected" };
    if proposal.status == target && token.state == target {
        return Ok(proposal);
    }
    if proposal.status != "pending" {
        return Err(format!("invalid_status: proposal is {}", proposal.status));
    }
    if auth::now_ms() > token.expires_at_ms {
        db.transition_approval(&token.id, "pending", "expired", auth::now_ms())?;
        db.update_promotion_status(proposal_id, "pending", "expired", auth::now_ms())?;
        return Err("approval_expired: promotion token expired".into());
    }
    if !db.transition_approval(&token.id, "pending", target, auth::now_ms())? {
        return Err("approval_mismatch: token transition failed".into());
    }
    if !db.update_promotion_status(proposal_id, "pending", target, auth::now_ms())? {
        return Err("approval_mismatch: proposal transition failed".into());
    }
    proposal.status = target.into();
    proposal.decided_at = Some(auth::now_ms());
    db.append_audit(
        auth::now_ms(),
        &operator.display_name,
        if approve { "promotion.approved" } else { "promotion.rejected" },
        &format!("{proposal_id} {}", token.id),
    )?;
    Ok(proposal)
}

#[tauri::command]
pub fn whatif_promote_approve(proposal_id: String, state: State<'_, AppState>) -> Result<PromotionProposal, String> {
    let operator = auth::require(&state.session)?;
    decide_promotion(true, &proposal_id, &state, &operator)
}

#[tauri::command]
pub fn whatif_promote_reject(proposal_id: String, state: State<'_, AppState>) -> Result<PromotionProposal, String> {
    let operator = auth::require(&state.session)?;
    decide_promotion(false, &proposal_id, &state, &operator)
}

#[tauri::command]
pub fn whatif_promote_cancel(proposal_id: String, state: State<'_, AppState>) -> Result<PromotionProposal, String> {
    let operator = auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let proposal_json =
        db.promotion_proposal(&proposal_id)?.ok_or_else(|| format!("unknown_proposal: {proposal_id}"))?;
    let mut proposal: PromotionProposal = serde_json::from_str(&proposal_json).map_err(|error| error.to_string())?;
    if proposal.status == "cancelled" {
        return Ok(proposal);
    }
    if proposal.status != "pending" && proposal.status != "approved" {
        return Err(format!("invalid_status: proposal is {}", proposal.status));
    }
    let token = token_for(&db, &proposal.approval_token_id)?;
    db.transition_approval(&token.id, &token.state, "cancelled", auth::now_ms())?;
    db.update_promotion_status(&proposal_id, &proposal.status, "cancelled", auth::now_ms())?;
    proposal.status = "cancelled".into();
    db.append_audit(
        auth::now_ms(),
        &operator.display_name,
        "promotion.cancelled",
        &format!("{proposal_id} {}", token.id),
    )?;
    Ok(proposal)
}

#[tauri::command]
pub fn whatif_promote_commit(proposal_id: String, state: State<'_, AppState>) -> Result<PromotionCommit, String> {
    let operator = auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    if let Some(existing) = db.promotion_commit(&proposal_id)? {
        return serde_json::from_str(&existing).map_err(|error| error.to_string());
    }
    let proposal_json =
        db.promotion_proposal(&proposal_id)?.ok_or_else(|| format!("unknown_proposal: {proposal_id}"))?;
    let mut proposal: PromotionProposal = serde_json::from_str(&proposal_json).map_err(|error| error.to_string())?;
    if let Some((status, _, _)) = db.promotion_status(&proposal_id)? {
        proposal.status = status;
    }
    if db.promotion_conflict(&proposal.proposal_id, &operator.display_name)? {
        return Err("recused: operator declared a conflict for this proposal".into());
    }
    if db.promotion_four_eyes()? && proposal.requested_by == operator.display_name {
        return Err("separation_of_duties: four-eyes policy requires a different approver".into());
    }
    if proposal.status != "approved" {
        return Err(format!("approval_required: proposal is {}", proposal.status));
    }
    mandate_allows(&proposal, auth::now_ms())?;
    let token = token_for(&db, &proposal.approval_token_id)?;
    if token.state != "approved" {
        return Err(format!("approval_mismatch: token is {}", token.state));
    }
    if auth::now_ms() > token.expires_at_ms {
        return Err("approval_expired: promotion token expired".into());
    }
    if proposal.actions.iter().any(|action| action.valid_until.0 <= auth::now_ms()) {
        return Err("approval_expired: proposed action window has expired".into());
    }
    let variant_json =
        db.whatif_variant(&proposal.variant_id)?.ok_or_else(|| format!("unknown_variant: {}", proposal.variant_id))?;
    let snapshot_json = db
        .whatif_snapshot(&proposal.snapshot_id)?
        .ok_or_else(|| format!("unknown_snapshot: {}", proposal.snapshot_id))?;
    let variant: Variant = serde_json::from_str(&variant_json).map_err(|error| error.to_string())?;
    let snapshot: Snapshot = serde_json::from_str(&snapshot_json).map_err(|error| error.to_string())?;
    snapshot.verify_model(&state.model)?;
    let variant_sha = poie_digest::sha256_of("poie.whatif-variant/0.1", &variant).map_err(|error| error.to_string())?;
    let result_json =
        db.whatif_result(&proposal.result_id)?.ok_or_else(|| format!("unknown_result: {}", proposal.result_id))?;
    let result: VariantResult = serde_json::from_str(&result_json).map_err(|error| error.to_string())?;
    let inputs = poie_whatif::promotion::promotion_inputs_sha256(
        &snapshot.snapshot_sha256,
        &variant_sha,
        &result.result_sha256,
    )?;
    if inputs != proposal.inputs_sha256 {
        return Err("stale_result: snapshot/variant/result changed since propose".into());
    }
    if !db.transition_approval(&token.id, "approved", "executing", auth::now_ms())? {
        return Err("approval_mismatch: token transition failed".into());
    }
    for action in &proposal.actions {
        let draft_sha = proposed_action_sha256(action)?;
        let evidence = PromotionEvidence {
            approval_token_id: token.id.clone(),
            proposal_sha256: proposal.proposal_sha256.clone(),
            basis: format!(
                "approval:{};simulation=whatif:{};model={}",
                token.id, snapshot.snapshot_sha256, snapshot.model_sha256
            ),
        };
        let proposed = ProposedAction {
            schema: PROPOSED_ACTION_SCHEMA.into(),
            schema_version: "0.1".into(),
            proposal_id: proposal.proposal_id.clone(),
            action_id: action.action_id.clone(),
            action_sha256: draft_sha.clone(),
            action: action.clone(),
            state: "proposed".into(),
            authorization: None,
            promotion_evidence: evidence,
        };
        db.save_proposed_action(
            &proposal.proposal_id,
            &action.action_id,
            &serde_json::to_string(&proposed).map_err(|error| error.to_string())?,
            &draft_sha,
            auth::now_ms(),
        )?;
    }
    let commit = PromotionCommit::create(
        &proposal,
        proposal.actions_sha256.clone(),
        auth::now_ms(),
        operator.display_name.clone(),
    )?;
    db.save_promotion_commit(
        &proposal.proposal_id,
        &serde_json::to_string(&commit).map_err(|error| error.to_string())?,
        &commit.commit_sha256,
        &proposal.inputs_sha256,
        &token.id,
        auth::now_ms(),
        &operator.display_name,
    )?;
    db.transition_approval(&token.id, "executing", "consumed", auth::now_ms())?;
    db.update_promotion_status(&proposal.proposal_id, "approved", "committed", auth::now_ms())?;
    db.append_audit(
        auth::now_ms(),
        &operator.display_name,
        "promotion.committed",
        &format!("{} {} {}", proposal.proposal_id, proposal.scope, commit.commit_sha256),
    )?;
    Ok(commit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_model_snapshot_variant_run_and_persist() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../scenarios/base-01/model.toml");
        let model = poie_ontology::model::Model::load(&path).unwrap();
        let digest = poie_objects::model_digest(&model).unwrap();
        let snapshot = Snapshot::create(
            poie_whatif::SnapshotInput {
                label: "base".into(),
                model_sha256: digest,
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
            "snap-accept".into(),
            1,
            "acceptance".into(),
        )
        .unwrap();
        let variant = Variant {
            schema: VARIANT_SCHEMA.into(),
            schema_version: "0.1".into(),
            id: "var-accept".into(),
            snapshot_id: snapshot.id.clone(),
            label: "pump failed".into(),
            description: String::new(),
            patch: PatchDto {
                asset_status: [("pump-p1".to_string(), "failed".to_string())].into_iter().collect(),
                ..Default::default()
            },
            goal: None,
            min_verification: None,
            deadline_s: None,
            created_at: 1,
            created_by: "acceptance".into(),
        };
        let result = run_structure(&model, &snapshot, &variant).unwrap();
        let db = crate::db::Db::open_in_memory().unwrap();
        db.save_whatif_snapshot(
            &snapshot.id,
            &snapshot.schema_version,
            "session",
            &serde_json::to_string(&snapshot).unwrap(),
            &snapshot.snapshot_sha256,
            1,
        )
        .unwrap();
        db.save_whatif_variant(
            &variant.id,
            &variant.snapshot_id,
            &variant.schema_version,
            &serde_json::to_string(&variant).unwrap(),
            1,
        )
        .unwrap();
        let settings = serde_json::to_string(&result.settings).unwrap();
        db.save_whatif_result(
            &result.result_id,
            &result.variant_id,
            &result.snapshot_id,
            &result.schema_version,
            &result.mode,
            &settings,
            &serde_json::to_string(&result).unwrap(),
            &result.result_sha256,
            1,
        )
        .unwrap();
        let loaded: VariantResult =
            serde_json::from_str(&db.whatif_result(&result.result_id).unwrap().unwrap()).unwrap();
        assert_eq!(loaded.result_sha256, result.result_sha256);
        assert!(loaded.outcome.structure.is_some());
    }

    #[test]
    fn promotion_replay_is_deterministic() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../scenarios/base-01/model.toml");
        let model = poie_ontology::model::Model::load(&path).unwrap();
        let digest = poie_objects::model_digest(&model).unwrap();
        let snapshot = Snapshot::create(
            poie_whatif::SnapshotInput {
                label: "base".into(),
                model_sha256: digest,
                evidence_sha256: None,
                goal: "restore:pump-p1".into(),
                min_verification: "demonstrated".into(),
                horizon_s: 86_400.0,
                now_s: 0.0,
                start_s: 0.0,
                deadline_s: None,
                max_depth: 12,
                max_actions: 12,
                seed: 7,
            },
            "snap-replay".into(),
            1,
            "acceptance".into(),
        )
        .unwrap();
        let variant = Variant {
            schema: VARIANT_SCHEMA.into(),
            schema_version: "0.1".into(),
            id: "var-replay".into(),
            snapshot_id: snapshot.id.clone(),
            label: "baseline".into(),
            description: String::new(),
            patch: PatchDto::default(),
            goal: None,
            min_verification: None,
            deadline_s: None,
            created_at: 1,
            created_by: "acceptance".into(),
        };
        let now = 1_800_000_000_000i64;
        let state_version = promotion_state_version();
        let first = plan_for_promotion(&model, &snapshot, &variant).unwrap();
        let second = plan_for_promotion(&model, &snapshot, &variant).unwrap();
        let first_sha = poie_whatif::promotion::promotion_plan_sha256(&first).unwrap();
        let second_sha = poie_whatif::promotion::promotion_plan_sha256(&second).unwrap();
        assert_eq!(first_sha, second_sha);
        let first_actions = plan_to_actions_at(&first, snapshot.now_s, now, &state_version).unwrap();
        let second_actions = plan_to_actions_at(&second, snapshot.now_s, now, &state_version).unwrap();
        assert_eq!(
            poie_whatif::promotion::proposed_actions_sha256(&first_actions).unwrap(),
            poie_whatif::promotion::proposed_actions_sha256(&second_actions).unwrap()
        );
    }

    #[test]
    fn baseline_plan_can_be_rebased_and_previewed_for_promotion() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../scenarios/base-01/model.toml");
        let model = poie_ontology::model::Model::load(&path).unwrap();
        let digest = poie_objects::model_digest(&model).unwrap();
        let snapshot = Snapshot::create(
            poie_whatif::SnapshotInput {
                label: "base".into(),
                model_sha256: digest,
                evidence_sha256: None,
                goal: "service:potable-water".into(),
                min_verification: "demonstrated".into(),
                horizon_s: 86_400.0,
                now_s: 0.0,
                start_s: 0.0,
                deadline_s: None,
                max_depth: 12,
                max_actions: 12,
                seed: 7,
            },
            "snap-promote".into(),
            1,
            "acceptance".into(),
        )
        .unwrap();
        let variant = Variant {
            schema: VARIANT_SCHEMA.into(),
            schema_version: "0.1".into(),
            id: "var-promote".into(),
            snapshot_id: snapshot.id.clone(),
            label: "baseline".into(),
            description: String::new(),
            patch: PatchDto::default(),
            goal: None,
            min_verification: None,
            deadline_s: None,
            created_at: 1,
            created_by: "acceptance".into(),
        };
        let planned = plan_for_promotion(&model, &snapshot, &variant).unwrap();
        assert!(!planned.actions.is_empty());
        let now = 1_800_000_000_000i64;
        let state_version = promotion_state_version();
        let actions = plan_to_actions_at(&planned, snapshot.now_s, now, &state_version).unwrap();
        assert_eq!(actions[0].valid_from.0, now);
    }

    #[test]
    fn four_eyes_only_allows_a_different_approver() {
        assert!(second_approver_allowed(true, "Ada", "Bob"));
        assert!(!second_approver_allowed(true, "Ada", "Ada"));
        assert!(!second_approver_allowed(false, "Ada", "Bob"));
    }
}
