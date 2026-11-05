//! Phase C1 what-if commands: static snapshots, Patch variants, compare.
//!
//! The source model is read-only here. Results are written only to the
//! `whatif_*` tables; no live/SAP/federation/approval path is touched.

use poie_sim::timed::{TimedCase, TimedRunner, TimedScenario};
use poie_whatif::timed::{
    compare_timed, TimedCompareResult, TimedResult, TimedSnapshot, TimedSnapshotInput, TimedVariant,
    TimedVariantInput, TIMED_SNAPSHOT_SCHEMA,
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
    let parsed: toml::Value =
        toml::from_str(&text).map_err(|error| format!("scenario_mismatch: {error}"))?;
    let model_relative = parsed
        .get("model")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| "scenario_mismatch: missing model".to_string())?;
    let parent = std::path::Path::new(scenario_path)
        .parent()
        .and_then(|path| path.to_str())
        .unwrap_or_default();
    let model_text =
        sandbox::read_inside(&state.sandbox_root, parent, model_relative).map_err(|error| format!("io_error: {error}"))?;
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
    Ok(TimedScenarioInfo {
        scenario_sha256,
        scenario_path: path,
        model_sha256,
        description: scenario.description,
    })
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
    db.save_timed_snapshot(
        &snapshot.id,
        TIMED_SNAPSHOT_SCHEMA,
        &json,
        &snapshot.snapshot_sha256,
        auth::now_ms(),
    )?;
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
    let case = TimedCase {
        goal,
        start_s,
        deadline_s,
        horizon_s: snapshot.horizon_s,
        min_verification,
    };
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
}
